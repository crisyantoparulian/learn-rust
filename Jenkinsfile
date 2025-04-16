pipeline {
  agent any

  environment {
    PROJECT_ID = 'learn-kubernetes-cris'      // Replace with your GCP project ID
    REGION = 'us-central1'                  // Adjust if you use a different region
    REPO_NAME = 'rust-artifacts'            // Replace with your Artifact Registry repo name
    IMAGE_NAME = 'learn-rust-app'           // Whatever you want to name the Docker image
    TAG = 'latest'
    FULL_IMAGE_NAME = "${REGION}-docker.pkg.dev/${PROJECT_ID}/${REPO_NAME}/${IMAGE_NAME}:${TAG}"
  }

  stages {
    stage('Checkout') {
      steps {
        checkout scm
      }
    }

    stage('Docker Build') {
      steps {
        script {
          sh """
            docker build -t ${FULL_IMAGE_NAME} .
          """
        }
      }
    }

    stage('Authenticate with GCP') {
      steps {
        withCredentials([file(credentialsId: 'gcp-sa-key', variable: 'GOOGLE_APPLICATION_CREDENTIALS')]) {
          sh 'gcloud auth activate-service-account --key-file=$GOOGLE_APPLICATION_CREDENTIALS'
          sh 'gcloud auth configure-docker ${REGION}-docker.pkg.dev --quiet'
        }
      }
    }

    stage('Push to Artifact Registry') {
      steps {
        sh "docker push ${FULL_IMAGE_NAME}"
      }
    }
  }

  post {
    success {
      echo "✅ Successfully built and pushed image: ${FULL_IMAGE_NAME}"
    }
    failure {
      echo "❌ Build failed"
    }
  }
}
