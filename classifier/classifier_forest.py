import json
import numpy as np
import re
import argparse
from sklearn.preprocessing import OneHotEncoder, LabelEncoder
from sklearn.model_selection import train_test_split
from sklearn.ensemble import RandomForestClassifier
from sklearn.metrics import classification_report

# Retrieves header names and values from response_text parameter
def parse_headers(response_text):
    parts = response_text.split('\r\n\r\n', 1)
    header_part = parts[0] if parts else ''
    headers = {}
    header_lines = re.findall(r'^(.*?): (.*?)(?=\r\n|$)', header_part, re.MULTILINE)
    for header, value in header_lines:
        headers[header] = {
            'value': value,
            'original': header
        }
    return headers

# Normalizes header names to lowercase
def normalize_header(header):
    return header.lower()

# Determines capitalization style of a given header
def get_capitalization_style(header):
    if header.isupper():
        return 'uppercase'
    elif header.islower():
        return 'lowercase'
    elif header.istitle():
        return 'titlecase'
    else:
        return 'other'

# Extract features from a single sample
def extract_features(response_data, all_headers, excluded_headers):
    header_index = {header: index for index, header in enumerate(all_headers)}
    
    response_headers = parse_headers(response_data['response'])
    
    response_lines = response_data.get('response', '').split('\r\n')
    status_line = response_lines[0] if response_lines else ''
    status_parts = status_line.split(' ', 2)
    status_code = status_parts[1] if len(status_parts) > 1 else ''
    status_message = status_parts[2] if len(status_parts) > 2 else ''
    
    header_presence_vector = np.zeros(len(all_headers), dtype=int)
    header_values = ['undefined'] * len(all_headers)
    header_capitalization = ['undefined'] * len(all_headers)
    
    for header in response_headers:
        header_lower = normalize_header(header)
        if header_lower in header_index and header_lower not in excluded_headers:
            idx = header_index[header_lower]
            header_presence_vector[idx] = 1
            header_values[idx] = response_headers[header]['value']
            header_capitalization[idx] = get_capitalization_style(header)
    
    feature_vector = {
        'header_presence': header_presence_vector.tolist(),
        'header_values': header_values,
        'header_capitalization': header_capitalization,
        'response_time': response_data.get('response_time', 0),
        'status_code': int(status_code) if status_code.isdigit() else 0,
        'status_message': status_message
    }
    
    return feature_vector

def main(input_file):
    with open(input_file) as f:
        data = json.load(f)

    excluded_headers = {'date'}

    # Gather header names
    all_headers_set = set()
    for data_point in data:
        headers = parse_headers(data_point['response'])
        for header in headers:
            normalized_header = normalize_header(header)
            if normalized_header not in excluded_headers:
                all_headers_set.add(normalized_header)

    all_headers = sorted(all_headers_set)

    # Extract features from each sample
    features = []
    labels = []
    header_values = []
    header_capitalizations = []

    for item in data:
        feature_vector = extract_features(item, all_headers, excluded_headers)
        features.append(feature_vector)
        labels.append(item['framework'])
        header_values.extend(feature_vector['header_values'])
        header_capitalizations.extend(feature_vector['header_capitalization'])

    # Obtain unique header values and capitalizations
    header_values = np.array(header_values)
    header_capitalizations = np.array(header_capitalizations)

    unique_values = np.unique(header_values)
    unique_capitalization_styles = np.unique(header_capitalizations)

    # Initialize encoders for header values and capitalizations
    header_value_encoder = OneHotEncoder(sparse_output=False, handle_unknown='ignore')
    header_value_encoder.fit(unique_values.reshape(-1, 1))

    header_cap_encoder = OneHotEncoder(sparse_output=False, handle_unknown='ignore')
    header_cap_encoder.fit(unique_capitalization_styles.reshape(-1, 1))

    # Setup features for the model
    X = []
    y = LabelEncoder().fit_transform(labels)

    for feature_vector in features:
        header_value_encoded = header_value_encoder.transform(np.array(feature_vector['header_values']).reshape(-1, 1))
        header_cap_encoded = header_cap_encoder.transform(np.array(feature_vector['header_capitalization']).reshape(-1, 1))
        
        combined_features = np.hstack([
            feature_vector['header_presence'],
            [feature_vector['response_time']],
            [feature_vector['status_code']],
            header_value_encoded.flatten(),
            header_cap_encoded.flatten()
        ])
    
        X.append(combined_features)

    X = np.array(X)

    # Divide samples into train and test sets
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.3, random_state=42)

    # Training
    model = RandomForestClassifier(n_estimators=100, random_state=42)
    model.fit(X_train, y_train)

    # Obtain metrics
    y_pred = model.predict(X_test)
    print(classification_report(y_test, y_pred))

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Train a Random Forest model based on JSON results obtained from the Wmap fuzzer module.')
    parser.add_argument('input_file', type=str, help='Path to the JSON file containing the HTTP response data.')
    args = parser.parse_args()
    
    main(args.input_file)
