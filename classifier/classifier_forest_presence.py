import json
import re
from typing import List, Dict, Any
import numpy as np
import matplotlib.pyplot as plt
from sklearn.feature_extraction.text import CountVectorizer
from sklearn.preprocessing import LabelEncoder
from sklearn.model_selection import train_test_split
from sklearn.ensemble import RandomForestClassifier
from sklearn.metrics import classification_report
import pandas as pd

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

def extract_features(sample: Dict[str, Any], all_headers: List[str], excluded_headers: List[str]) -> Dict[str, Any]:
    response = sample["response"]
    headers = extract_headers(response, excluded_headers)
    status_code, status_message = extract_status(response)
    
    features = {
        "header_presence": [int(header.lower() in headers) for header in all_headers],
        "response_time": sample["response_time"],
        "status_code": status_code,
        "status_message": status_message
    }
    
    for header in all_headers:
        features[f"{header}_value"] = headers.get(header.lower(), "undefined")
        features[f"{header}_capitalization"] = get_capitalization(headers.get(header, ""))
    
    return features

def get_capitalization(s: str) -> str:
    if s.islower():
        return "lowercase"
    elif s.isupper():
        return "uppercase"
    elif s.istitle():
        return "titlecase"
    else:
        return "other"

def prepare_data(samples: List[Dict[str, Any]], excluded_headers: List[str]) -> tuple:
    all_headers = set()
    for sample in samples:
        headers = extract_headers(sample["response"], excluded_headers)
        all_headers.update(headers.keys())
    all_headers = sorted(list(all_headers))

    X = [extract_features(sample, all_headers, excluded_headers) for sample in samples]
    y = [sample["framework"] for sample in samples]

    return X, y, all_headers

def vectorize_features(X: List[Dict[str, Any]], all_headers: List[str]) -> tuple:
    header_presence = np.array([x["header_presence"] for x in X])

    numeric_features = np.array([[x["response_time"], x["status_code"]] for x in X])
    
    categorical_features = []
    feature_names = ["Header Presence"] * len(all_headers) + ["Response Time", "Status Code"]
    
    for header in all_headers:
        le_value = LabelEncoder()
        le_cap = LabelEncoder()
        value_feature = le_value.fit_transform([x[f"{header}_value"] for x in X])
        cap_feature = le_cap.fit_transform([x[f"{header}_capitalization"] for x in X])
        categorical_features.extend([value_feature.reshape(-1, 1), cap_feature.reshape(-1, 1)])
        feature_names.extend([f"{header} Value", f"{header} Casing"])
    
    status_message_le = LabelEncoder()
    status_message = status_message_le.fit_transform([x["status_message"] for x in X])
    categorical_features.append(status_message.reshape(-1, 1))
    feature_names.append("Status Message")
    
    return np.hstack([header_presence, numeric_features] + categorical_features), feature_names

def train_classifier(X: np.ndarray, y: List[str]):
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)
    
    clf = RandomForestClassifier(n_estimators=100, random_state=42)
    clf.fit(X_train, y_train)
    
    y_pred = clf.predict(X_test)
    print(classification_report(y_test, y_pred))
    
    return clf

def plot_feature_importance(clf: RandomForestClassifier, feature_names: List[str]):
    importances = clf.feature_importances_
    feature_importance = {}

    for importance, name in zip(importances, feature_names):
        print(f"{name}: {importance}")
        if name in ["Status Code", "Status Message", "Response Time"]:
            feature_importance[name] = importance
        elif "Header Order" in name:
            feature_importance["Header Order"] = feature_importance.get("Header Order", 0) + importance
        elif "Header Presence" in name:
            feature_importance["Header Presence"] = feature_importance.get("Header Presence", 0) + importance
        else:  # Specific header value or casing
            feature_importance[name] = importance
    
    # Sort features by importance
    sorted_features = sorted(feature_importance.items(), key=lambda x: x[1], reverse=True)
    
    # Plot top 20 features
    top_features = sorted_features[:20]
    features, importances = zip(*top_features)
    
    plt.figure(figsize=(12, 8))
    plt.bar(range(len(importances)), importances)
    plt.xticks(range(len(importances)), features, rotation=90)
    plt.title("Top 20 Most Important Features")
    plt.xlabel("Features")
    plt.ylabel("Importance")
    plt.tight_layout()
    plt.savefig("feature_importance.png")
    print("Feature importance graph saved as 'feature_importance.png'")

def main():
    file_path = "results.json"
    excluded_headers = ["date", "server"]
    samples = load_data(file_path)
    X, y, all_headers = prepare_data(samples, excluded_headers)
    X_vectorized, feature_names = vectorize_features(X, all_headers)
    classifier = train_classifier(X_vectorized, y)
    plot_feature_importance(classifier, feature_names)

if __name__ == "__main__":
    main()