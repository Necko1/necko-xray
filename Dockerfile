FROM ubuntu:latest
LABEL authors="necko"

ENTRYPOINT ["top", "-b"]