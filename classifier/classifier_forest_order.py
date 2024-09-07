import json
import re
from typing import List, Dict, Any
import numpy as np
import matplotlib.pyplot as plt
from sklearn.feature_extraction.text import CountVectorizer
from sklearn.preprocessing import LabelEncoder
from sklearn.metrics import classification_report
from sklearn.model_selection import train_test_split
from sklearn.ensemble import RandomForestClassifier
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

    header_order = [all_headers.index(header) if header in all_headers else -1 for header in all_headers]

    features = {
        "header_order": header_order,
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

from typing import List, Dict, Any
import numpy as np
from sklearn.preprocessing import OneHotEncoder
from sklearn.impute import SimpleImputer

def vectorize_features(X: List[Dict[str, Any]], all_headers: List[str]) -> tuple:
    header_order = np.array([x["header_order"] for x in X])
    numeric_features = np.array([[x["response_time"], x["status_code"]] for x in X])
    categorical_features = []
    feature_names = ["Header Order"] * header_order.shape[1] + ["Response Time", "Status Code"]
    encoders = {}

    for header in all_headers:
        value_feature = [x[f"{header}_value"] for x in X]
        cap_feature = [x[f"{header}_capitalization"] for x in X]
        
        value_encoder = OneHotEncoder(sparse_output=False, handle_unknown='ignore')
        cap_encoder = OneHotEncoder(sparse_output=False, handle_unknown='ignore')
        
        value_encoded = value_encoder.fit_transform(np.array(value_feature).reshape(-1, 1))
        cap_encoded = cap_encoder.fit_transform(np.array(cap_feature).reshape(-1, 1))
        
        categorical_features.extend([value_encoded, cap_encoded])
        feature_names.extend([f"{header} Value {val}" for val in value_encoder.categories_[0]])
        feature_names.extend([f"{header} Casing {val}" for val in cap_encoder.categories_[0]])
        
        encoders[f"{header}_value"] = value_encoder
        encoders[f"{header}_capitalization"] = cap_encoder

    status_message_encoder = OneHotEncoder(sparse_output=False, handle_unknown='ignore')
    status_message = status_message_encoder.fit_transform(np.array([x["status_message"] for x in X]).reshape(-1, 1))
    categorical_features.append(status_message)
    feature_names.extend([f"Status Message {val}" for val in status_message_encoder.categories_[0]])
    encoders["status_message"] = status_message_encoder

    # Combine all features
    all_features = np.hstack([header_order, numeric_features] + categorical_features)

    # Handle missing values
    imputer = SimpleImputer(strategy='constant', fill_value=0)
    all_features = imputer.fit_transform(all_features)

    return all_features, feature_names, encoders

def train_classifier(X: np.ndarray, y: List[str], model_path: str, all_headers: List[str], le: LabelEncoder):
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)
    
    clf = RandomForestClassifier(n_estimators=100, random_state=42)
    clf.fit(X_train, y_train)
    
    y_pred = clf.predict(X_test)
    
    # Convert numeric labels back to framework names for the classification report
    y_test_names = le.inverse_transform(y_test)
    y_pred_names = le.inverse_transform(y_pred)
    
    print(classification_report(y_test_names, y_pred_names))
    
    # Save the trained model, all_headers, and LabelEncoder
    with open(model_path, "wb") as f:
        pickle.dump((clf, all_headers, le), f)
    
    return clf

def load_model(model_path: str):
    with open(model_path, "rb") as f:
        model, all_headers, le = pickle.load(f)
    return model, all_headers, le

def predict(X: np.ndarray, model: RandomForestClassifier, le: LabelEncoder):
    y_pred = model.predict(X)
    print("Predicted frameworks:")
    for prediction in y_pred:
        try:
            print(le.inverse_transform([prediction])[0])
        except ValueError:
            print(f"Unknown framework (label {prediction})")

def main():
    parser = argparse.ArgumentParser(description="Train and predict web frameworks")
    parser.add_argument("--mode", choices=["train", "predict"], required=True, help="Mode of operation (train or predict)")
    parser.add_argument("--file", type=str, required=True, help="Path to the input JSON file")
    parser.add_argument("--model", type=str, default="model.pkl", help="Path to save/load the trained model")
    args = parser.parse_args()

    if args.mode == "train":
        samples = load_data(args.file)
        X, y, all_headers = prepare_data(samples, [], ["date", "server"])
        X_vectorized, feature_names, le = vectorize_features(X, all_headers)
        le_framework = LabelEncoder()
        y_encoded = le_framework.fit_transform(y)
        classifier = train_classifier(X_vectorized, y_encoded, args.model, all_headers, le_framework)
        print(f"Model, headers, and label encoder saved to: {args.model}")
    elif args.mode == "predict":
        samples = load_data(args.file)
        model, saved_headers, le_framework = load_model(args.model)
        X, _, _ = prepare_data(samples, saved_headers, ["date", "server"])
        X_vectorized, feature_names, _ = vectorize_features(X, saved_headers)
        predict(X_vectorized, model, le_framework)

if __name__ == "__main__":
    main()