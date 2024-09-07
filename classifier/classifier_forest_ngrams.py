import json
import re
from typing import List, Dict, Any
import numpy as np
from sklearn.feature_extraction.text import CountVectorizer
from sklearn.preprocessing import LabelEncoder
from sklearn.model_selection import train_test_split
from sklearn.ensemble import RandomForestClassifier
from sklearn.metrics import classification_report
import matplotlib.pyplot as plt

def extract_headers(response: str, excluded_headers: List[str]) -> List[str]:
    headers = []
    header_pattern = r'(\S+):\s*(.*?)\r\n'
    matches = re.findall(header_pattern, response)
    for header, _ in matches:
        header_lower = header.lower()
        if header_lower not in excluded_headers:
            headers.append(header_lower)
    return headers

def extract_status(response: str) -> tuple:
    status_pattern = r'HTTP/\d\.\d (\d+) (.+)\r\n'
    match = re.search(status_pattern, response)
    if match:
        return int(match.group(1)), match.group(2)
    return None, None

def extract_features(sample: Dict[str, Any], excluded_headers: List[str]) -> Dict[str, Any]:
    response = sample['response']
    headers = extract_headers(response, excluded_headers)
    status_code, status_message = extract_status(response)
    
    features = {
        'header_order': ' '.join(headers),
        'response_time': sample['response_time'],
        'status_code': status_code,
        'status_message': status_message
    }
    
    return features

def prepare_data(samples: List[Dict[str, Any]], excluded_headers: List[str]) -> tuple:
    X = [extract_features(sample, excluded_headers) for sample in samples]
    y = [sample['framework'] for sample in samples]
    return X, y

def vectorize_features(X: List[Dict[str, Any]]) -> tuple:
    header_order_vectorizer = CountVectorizer(analyzer="word", tokenizer=lambda x: x.split(), ngram_range=(1, 3))
    header_order = header_order_vectorizer.fit_transform([x['header_order'] for x in X]).toarray()

    numeric_features = np.array([[x['response_time'], x['status_code']] for x in X])
    
    status_message_le = LabelEncoder()
    status_message = status_message_le.fit_transform([x['status_message'] for x in X]).reshape(-1, 1)
    
    feature_names = (
        [f'Header Order {i}' for i in range(header_order.shape[1])] +
        ['Response Time', 'Status Code', 'Status Message']
    )
    
    return np.hstack([header_order, numeric_features, status_message]), feature_names

def train_classifier(X: np.ndarray, y: List[str], feature_names: List[str]):
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)
    
    clf = RandomForestClassifier(n_estimators=100, random_state=42)
    clf.fit(X_train, y_train)
    
    y_pred = clf.predict(X_test)
    print(classification_report(y_test, y_pred))
    
    return clf, feature_names

def plot_feature_importance(clf: RandomForestClassifier, feature_names: List[str]):
    importances = clf.feature_importances_
    feature_importance = {}
    
    for importance, name in zip(importances, feature_names):
        if name.startswith('Header Order'):
            feature_importance['Header Order'] = feature_importance.get('Header Order', 0) + importance
        else:
            feature_importance[name] = importance
    
    sorted_features = sorted(feature_importance.items(), key=lambda x: x[1], reverse=True)
    top_features = sorted_features[:20]
    features, importances = zip(*top_features)
    
    plt.figure(figsize=(12, 8))
    plt.bar(range(len(importances)), importances)
    plt.xticks(range(len(importances)), features, rotation=90)
    plt.title('Top 20 Most Important Features')
    plt.xlabel('Features')
    plt.ylabel('Importance')
    plt.tight_layout()
    plt.savefig('feature_importance.png')
    print("Feature importance graph saved as 'feature_importance.png'")

def main():
    file_path = 'results.json'
    excluded_headers = {"date"}
    
    with open(file_path, 'r') as f:
        samples = json.load(f)
    
    X, y = prepare_data(samples, excluded_headers)
    X_vectorized, feature_names = vectorize_features(X)
    classifier, feature_names = train_classifier(X_vectorized, y, feature_names)
    plot_feature_importance(classifier, feature_names)

if __name__ == "__main__":
    main()