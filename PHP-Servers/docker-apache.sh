#!/bin/sh

if [ ! `which docker` ]; then
    echo "docker not found."
    exit 1;
fi

PORT=${1:-8080}

echo "Building Docker image for Apache..."
docker-compose build

echo "Runing Apache in http://127.0.0.1:$PORT"
echo -e "Press Ctrl+C to stop. \n\n"

docker-compose up
