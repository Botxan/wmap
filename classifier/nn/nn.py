import argparse
import json
import re
from typing import List, Dict, Any
import numpy as np
from sklearn.preprocessing import LabelEncoder, OneHotEncoder
from sklearn.model_selection import train_test_split
from sklearn.metrics import classification_report, confusion_matrix
from sklearn.impute import SimpleImputer
from tensorflow.keras.models import Model, load_model
from tensorflow.keras.layers import Input, Embedding, LSTM, Dense, Concatenate, Dropout
from tensorflow.keras.callbacks import EarlyStopping, ReduceLROnPlateau
import pickle
import matplotlib.pyplot as plt
import seaborn as sns

def load_data(file_path: str) -> List[Dict[str, Any]]:
    with open(file_path, 'r') as f:
        data = json.load(f)
    return data

def extract_headers(response: str, excluded_headers: List[str]) -> Dict[str, str]:
    headers = {}
    header_pattern = r"(\S+):\s*(.*?)\r\n"
    matches = re.findall(header_pattern, response)
    for header, value in matches:
        header_lower = header.lower()
        if header_lower not in excluded_headers:
            headers[header_lower] = value
    return headers

def extract_status(response: str) -> tuple:
    status_pattern = r"HTTP/\d\.\d (\d+) (.+)\r\n"
    match = re.search(status_pattern, response)
    if match:
        return int(match.group(1)), match.group(2)
    return None, None

def get_capitalization(s: str) -> str:
    if s.islower():
        return "lowercase"
    elif s.isupper():
        return "uppercase"
    elif s.istitle():
        return "titlecase"
    else:
        return "other"
    
def extract_features(sample: Dict[str, Any], all_headers: List[str], excluded_headers: List[str]) -> Dict[str, Any]:
    response = sample["response"]
    headers = extract_headers(response, excluded_headers)
    status_code, status_message = extract_status(response)

    header_order = [all_headers.index(header) if header in all_headers else -1 for header in headers]
    header_order += [-1] * (len(all_headers) - len(header_order))

    features = {
        "header_order": header_order,
        "response_time": sample["response_time"],
        "status_code": status_code,
        "status_message": status_message
    }

    for header in all_headers:
        if header in headers:
            features[f"{header}_value"] = headers[header]
            features[f"{header}_capitalization"] = get_capitalization(headers[header])
        else:
            features[f"{header}_value"] = "unknown"
            features[f"{header}_capitalization"] = "other"

    return features

def prepare_data(samples: List[Dict[str, Any]], all_headers: List[str], excluded_headers: List[str]) -> tuple:
    if not all_headers:
        all_headers = set()
        for sample in samples:
            headers = extract_headers(sample["response"], excluded_headers)
            all_headers.update(headers.keys())
        all_headers = sorted(list(all_headers))

    X = [extract_features(sample, all_headers, excluded_headers) for sample in samples]
    y = [sample["framework"] for sample in samples]

    return X, y, all_headers

def vectorize_features(X: List[Dict[str, Any]], all_headers: List[str], encoders=None, mode='fit'):
    header_order = np.array([x["header_order"] for x in X])
    numeric_features = np.array([[x["response_time"], x["status_code"]] for x in X])
    categorical_features = []
    feature_names = ["Header Order"] * header_order.shape[1] + ["Response Time", "Status Code"]

    if mode == 'fit':
        encoders = {}

    for header in all_headers:
        value_feature = [x[f"{header}_value"] for x in X]
        cap_feature = [x[f"{header}_capitalization"] for x in X]
        
        if mode == 'fit':
            value_encoder = OneHotEncoder(sparse_output=False, handle_unknown='ignore')
            cap_encoder = OneHotEncoder(sparse_output=False, handle_unknown='ignore')
            value_encoded = value_encoder.fit_transform(np.array(value_feature).reshape(-1, 1))
            cap_encoded = cap_encoder.fit_transform(np.array(cap_feature).reshape(-1, 1))
            encoders[f"{header}_value"] = value_encoder
            encoders[f"{header}_capitalization"] = cap_encoder
        else:
            value_encoder = encoders[f"{header}_value"]
            cap_encoder = encoders[f"{header}_capitalization"]
            value_encoded = value_encoder.transform(np.array(value_feature).reshape(-1, 1))
            cap_encoded = cap_encoder.transform(np.array(cap_feature).reshape(-1, 1))

        categorical_features.extend([value_encoded, cap_encoded])
        feature_names.extend([f"{header} Value {val}" for val in value_encoder.categories_[0]])
        feature_names.extend([f"{header} Casing {val}" for val in cap_encoder.categories_[0]])

    if mode == 'fit':
        status_message_encoder = OneHotEncoder(sparse_output=False, handle_unknown='ignore')
        status_message = status_message_encoder.fit_transform(np.array([x["status_message"] for x in X]).reshape(-1, 1))
        encoders["status_message"] = status_message_encoder
    else:
        status_message_encoder = encoders["status_message"]
        status_message = status_message_encoder.transform(np.array([x["status_message"] for x in X]).reshape(-1, 1))

    categorical_features.append(status_message)
    feature_names.extend([f"Status Message {val}" for val in status_message_encoder.categories_[0]])


    imputer = SimpleImputer(strategy='constant', fill_value=0)
    categorical_features = np.hstack(categorical_features)
    categorical_features = imputer.fit_transform(categorical_features)

    return header_order, numeric_features, categorical_features, feature_names, encoders

def create_model(header_vocab_size, max_header_length, embedding_dim, num_numeric, num_categorical, num_classes):
    header_input = Input(shape=(max_header_length,), name='header_input')
    numeric_input = Input(shape=(num_numeric,), name='numeric_input')
    categorical_input = Input(shape=(num_categorical,), name='categorical_input')

    embedding = Embedding(input_dim=header_vocab_size, output_dim=embedding_dim, input_length=max_header_length)(header_input)
    lstm = LSTM(50)(embedding)

    concatenated = Concatenate()([lstm, numeric_input, categorical_input])

    x = Dense(128, activation='relu')(concatenated)
    x = Dropout(0.3)(x)
    x = Dense(64, activation='relu')(x)
    x = Dropout(0.3)(x)
    outputs = Dense(num_classes, activation='softmax')(x)

    model = Model(inputs=[header_input, numeric_input, categorical_input], outputs=outputs)
    model.compile(optimizer='adam', loss='sparse_categorical_crossentropy', metrics=['accuracy'])
    
    return model

def train_classifier(header_order: np.ndarray, numeric_features: np.ndarray, categorical_features: np.ndarray, 
                y: np.ndarray, model_path: str, all_headers: List[str], encoders: Dict, le: LabelEncoder):
    
    header_vocab_size = len(all_headers) + 1  # +1 for unknown/padding
    max_header_length = header_order.shape[1]
    embedding_dim = 50
    num_numeric = numeric_features.shape[1]
    num_categorical = categorical_features.shape[1]
    num_classes = len(le.classes_)

    X_header_train, X_header_test, X_num_train, X_num_test, X_cat_train, X_cat_test, y_train, y_test = train_test_split(
        header_order, numeric_features, categorical_features, y, test_size=0.2, random_state=42
    )
    
    model = create_model(header_vocab_size, max_header_length, embedding_dim, num_numeric, num_categorical, num_classes)
    
    early_stopping = EarlyStopping(monitor='val_loss', patience=5, restore_best_weights=True)
    reduce_lr = ReduceLROnPlateau(monitor='val_loss', factor=0.2, patience=3, min_lr=0.001)
    
    history = model.fit(
        [X_header_train, X_num_train, X_cat_train],
        y_train,
        epochs=100,
        batch_size=32,
        validation_split=0.2,
        callbacks=[early_stopping, reduce_lr]
    )

    plot_training_history(history, model_path)
    
    y_pred = model.predict([X_header_test, X_num_test, X_cat_test])
    y_pred_classes = np.argmax(y_pred, axis=1)
    
    print(classification_report(y_test, y_pred_classes, target_names=le.classes_))

    plot_confusion_matrix(y_test, y_pred_classes, le.classes_, model_path)
    
    model.save(f"{model_path}.h5")
    with open(f"{model_path}_objects.pkl", "wb") as f:
        pickle.dump((all_headers, encoders, le), f)
    
    return model

def load_saved_model(model_path: str):
    model = load_model(f"{model_path}.h5")
    with open(f"{model_path}_objects.pkl", "rb") as f:
        all_headers, encoders, le = pickle.load(f)
    return model, all_headers, encoders, le

def predict(model: Model, header_order: np.ndarray, numeric_features: np.ndarray, categorical_features: np.ndarray, le: LabelEncoder):
    y_pred = model.predict([header_order, numeric_features, categorical_features])
    y_pred_classes = np.argmax(y_pred, axis=1)
    predicted_frameworks = le.inverse_transform(y_pred_classes)
    
    print("Predicted frameworks:")
    for framework in predicted_frameworks:
        print(framework)

def plot_confusion_matrix(y_true: np.ndarray, y_pred: np.ndarray, classes: List[str], output_file: str):
    cm = confusion_matrix(y_true, y_pred)
    plt.figure(figsize=(10, 8))
    sns.heatmap(cm, annot=True, fmt='d', cmap='Blues', xticklabels=classes, yticklabels=classes)
    plt.title('Confusion Matrix')
    plt.xlabel('Predicted')
    plt.ylabel('True')
    plt.tight_layout()
    plt.savefig(f"{output_file}_confusion_matrix.png")
    plt.close()

def plot_training_history(history, output_file: str):
    plt.figure(figsize=(12, 4))
    
    plt.subplot(1, 2, 1)
    plt.plot(history.history['loss'], label='Training Loss')
    plt.plot(history.history['val_loss'], label='Validation Loss')
    plt.title('Model Loss')
    plt.xlabel('Epoch')
    plt.ylabel('Loss')
    plt.legend()
    
    plt.subplot(1, 2, 2)
    plt.plot(history.history['accuracy'], label='Training Accuracy')
    plt.plot(history.history['val_accuracy'], label='Validation Accuracy')
    plt.title('Model Accuracy')
    plt.xlabel('Epoch')
    plt.ylabel('Accuracy')
    plt.legend()
    
    plt.tight_layout()
    plt.savefig(f"{output_file}_training_history.png")
    plt.close()

def main():
    parser = argparse.ArgumentParser(description="Train and predict web frameworks")
    parser.add_argument("--mode", choices=["train", "predict"], required=True, help="Mode of operation (train or predict)")
    parser.add_argument("--file", type=str, required=True, help="Path to the input JSON file")
    parser.add_argument("--model", type=str, default="model", help="Path to save/load the trained model (without extension)")
    args = parser.parse_args()

    excluded_headers = ["date", "server"]

    if args.mode == "train":
        samples = load_data(args.file)
        X, y, all_headers = prepare_data(samples, [], excluded_headers)
        header_order, numeric_features, categorical_features, feature_names, encoders = vectorize_features(X, all_headers, mode='fit')
        le = LabelEncoder()
        y_encoded = le.fit_transform(y)
        model = train_classifier(header_order, numeric_features, categorical_features, y_encoded, args.model, all_headers, encoders, le)
        print(f"Model and associated objects saved to {args.model}.h5 and {args.model}_objects.pkl")
        
    elif args.mode == "predict":
        model, all_headers, encoders, le = load_saved_model(args.model)
        samples = load_data(args.file)
        X, _, _ = prepare_data(samples, all_headers, excluded_headers)
        header_order, numeric_features, categorical_features, _, _ = vectorize_features(X, all_headers, encoders, mode='transform')
        predict(model, header_order, numeric_features, categorical_features, le)

if __name__ == "__main__":
    main()