# Define versions used to select image versions
# (ARGs declared before FROM can't be used outside of FROMs)
ARG RUST_VERSION=1.78

FROM rust:${RUST_VERSION}

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    build-essential \
    libssl-dev \
    pkg-config \
    cmake \
    curl \
    git

# Setup the work directory
WORKDIR /code

# Copy the project files
COPY . .

# Build the project
CMD ["bash"]
