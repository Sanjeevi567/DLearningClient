# Deep Learning Application with Amazon Rekognition, Polly, and Transcribe in Rust - CLI Guide

## Overview

This README guide provides step-by-step instructions on how to build a deep learning application using Amazon Rekognition, Polly, and Transcribe with Rust through the CLI.

**Table of Contents**

- [Deep Learning Application with Amazon Rekognition, Polly, and Transcribe in Rust - CLI Guide](#deep-learning-application-with-amazon-rekognition-polly-and-transcribe-in-rust---cli-guide)
  - [Overview](#overview)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [AWS CLI Application](#aws-cli-application)
  - [Examples and Documentation](#examples-and-documentation)

## Prerequisites

Before you begin, ensure you have the following prerequisites:

- [Rust](https://www.rust-lang.org/) installed.
- AWS CLI configured with access to Rekognition, Polly, and Transcribe services.

## Installation

1. Clone this repository:

   ```bash
   git clone https://github.com/Sanjuvi/DLearningClient.git
   cd DLearningClient
   ```

2. Build the application:

   ```bash
   cargo build --release
   ```

3. Set up AWS credentials:

Configure your AWS credentials using the AWS CLI:

```bash
aws configure
```
Follow the prompts to enter your AWS Access Key ID, Secret Access Key, region, and output format.

# AWS CLI Application

This CLI application interacts with various AWS services by utilizing the high-level APIs provided by the [aws_apis](https://github.com/Sanjuvi/aws_apis) repository. The [aws_apis](https://github.com/Sanjuvi/aws_apis) repository encompasses a range of AWS services, including SES (Simple Email Service), RDS (Relational Database Service), S3(Simple Storage Service) and MemoryDB.
## Examples and Documentation

For detailed instructions and explanations, please refer to our [comprehensive blog post](https://sanjuvi.github.io/Blog/posts/Deep-Learning-Rust/) that covers the complete usage of this application.

