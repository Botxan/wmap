import json
import re
from typing import List, Dict, Any
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from sklearn.preprocessing import LabelEncoder, OneHotEncoder
from sklearn.metrics import classification_report, confusion_matrix
from sklearn.feature_selection import mutual_info_classif
from sklearn.model_selection import train_test_split
from sklearn.ensemble import RandomForestClassifier
from sklearn.impute import SimpleImputer
import pickle
import argparse

def load_data(file_path: str) -> List[Dict[str, Any]]:
    with open(file_path, "r") as f:
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

    header_presence = [1 if header in headers else 0 for header in all_headers]

    features = {
        "header_presence": header_presence,
        "response_time": sample["response_time"],
        "status_code": status_code,
        "status_message": status_message
    }

    for header in all_headers:
        if header in headers:
            features[f"{header}_value"] = headers[header.lower()]
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

def vectorize_features(X: List[Dict[str, Any]], all_headers: List[str], mode='fit', encoders=None) -> tuple:
    header_presence = np.array([x["header_presence"] for x in X])
    numeric_features = np.array([[x["response_time"], x["status_code"]] for x in X])

    categorical_features = []
    feature_names = ["Header Presence"] * len(all_headers) + ["Response Time", "Status Code"]

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

    categorical_features = np.hstack(categorical_features)

    imputer = SimpleImputer(strategy='constant', fill_value=0)
    all_features = np.hstack([header_presence, numeric_features, categorical_features])
    all_features = imputer.fit_transform(all_features)

    return all_features, feature_names, encoders

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

def train_classifier(X: np.ndarray, y: List[str], model_path: str, all_headers: List[str], le: LabelEncoder, encoders: Dict, feature_names: List[str], samples: List[Dict[str, Any]]) -> RandomForestClassifier:
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)
    
    clf = RandomForestClassifier(n_estimators=100, random_state=22)
    clf.fit(X_train, y_train)
    
    y_pred = clf.predict(X_test)
    
    y_test_names = le.inverse_transform(y_test)
    y_pred_names = le.inverse_transform(y_pred)
    
    print(classification_report(y_test_names, y_pred_names))
    
    plot_confusion_matrix(y_test_names, y_pred_names, le.classes_, model_path)
    
    plot_feature_importance(clf, feature_names, model_path)
    
    plot_information_gain(X, y, feature_names, model_path)
     
    with open(model_path, "wb") as f:
        pickle.dump((clf, all_headers, le, encoders), f)
    
    return clf

def load_model(model_path: str) -> tuple:
    with open(model_path, "rb") as f:
        model, all_headers, le, encoders = pickle.load(f)
    return model, all_headers, le, encoders

def predict(X: np.ndarray, model: RandomForestClassifier, le: LabelEncoder):
    y_pred = model.predict(X)
    print("Predicted frameworks:")
    for prediction in y_pred:
        try:
            print(le.inverse_transform([prediction])[0])
        except ValueError:
            print(f"Unknown framework (label {prediction})")

def aggregate_header_order(feature_names: List[str], importances: np.ndarray) -> tuple:
    header_order_indices = [i for i, name in enumerate(feature_names) if name.startswith("Header Presence")]
    header_order_importance = np.sum(importances[header_order_indices])
    
    new_feature_names = [name for i, name in enumerate(feature_names) if i not in header_order_indices]
    new_importances = [imp for i, imp in enumerate(importances) if i not in header_order_indices]
    
    new_feature_names.append("Header Presence")
    new_importances.append(header_order_importance)
    
    return new_feature_names, np.array(new_importances)

def plot_top_features(feature_names: List[str], importances: np.ndarray, title: str, output_file: str):
    feature_names, importances = aggregate_header_order(feature_names, importances)
    
    indices = np.argsort(importances)[::-1]
    top_indices = indices[:10]
    
    plt.figure(figsize=(12, 8))
    plt.title(title)
    plt.bar(range(10), importances[top_indices])
    plt.xticks(range(10), [feature_names[i] for i in top_indices], rotation=90)
    plt.tight_layout()
    plt.savefig(output_file)
    plt.close()

def plot_feature_importance(clf: RandomForestClassifier, feature_names: List[str], output_file: str):
    importances = clf.feature_importances_
    plot_top_features(feature_names, importances, "Top 10 Feature Importances", f"{output_file}_top10_feature_importance.png")

def plot_information_gain(X: np.ndarray, y: np.ndarray, feature_names: List[str], output_file: str):
    ig = mutual_info_classif(X, y)
    plot_top_features(feature_names, ig, "Top 10 Information Gain", f"{output_file}_top10_information_gain.png")

def main():
    parser = argparse.ArgumentParser(description="Train and predict web frameworks")
    parser.add_argument("--mode", choices=["train", "predict"], required=True, help="Mode of operation (train or predict)")
    parser.add_argument("--file", type=str, required=True, help="Path to the input JSON file")
    parser.add_argument("--model", type=str, default="model.pkl", help="Path to save/load the trained model")
    args = parser.parse_args()

    excluded_headers = ["date", "server"]
    samples = load_data(args.file)

    if args.mode == "train":
        X, y, all_headers = prepare_data(samples, [], excluded_headers)
        X_vectorized, feature_names, encoders = vectorize_features(X, all_headers, mode='fit')
        le_framework = LabelEncoder()
        y_encoded = le_framework.fit_transform(y)
        train_classifier(X_vectorized, y_encoded, args.model, all_headers, le_framework, encoders, feature_names, samples)
        print(f"Model, headers, encoders, and label encoder saved to: {args.model}")
        
    elif args.mode == "predict":
        model, saved_headers, le_framework, encoders = load_model(args.model)
        X, _, _ = prepare_data(samples, saved_headers, excluded_headers)
        X_vectorized, _, _ = vectorize_features(X, saved_headers, mode='transform', encoders=encoders)
        predict(X_vectorized, model, le_framework)

if __name__ == "__main__":
    main()