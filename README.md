# 🌍 Climate Backend (Rust)

This is the backend service for the **Climate Time Machine / Global Heat Explorer** project, written in **Rust**.  
It provides APIs and data processing utilities for handling climate and temperature datasets.

---

## 🚀 Features
- Backend implemented in **Rust**
- Climate data fetching and preprocessing (ERA5, MODIS, etc.)
- REST API for frontend integration (React/Map-based apps)
- Handles geospatial processing and heatmap generation
- Secure handling of credentials with `service_account.json`

---

## 📦 Requirements
- [Rust & Cargo](https://www.rust-lang.org/tools/install) (Rust 1.70+ recommended)
- [Git](https://git-scm.com/)
- Google Cloud account (for accessing climate datasets)

---

## ⚙️ Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/amna55/ClimateTimeMachine_Rust.git
   cd ClimateTimeMachine_Rust
2. **Install dependencies**
   Rust dependencies are managed with Cargo. To fetch and build:
    ```cargo build
    cargo run
3. **Create service_account.json**
   Go to Google Cloud Console
   Create a Service Account and download the credentials as a JSON file.
   Save it in the root of this project as:
   service_account.json


⚠️ Important: Do not commit this file to GitHub.
It should only exist on your local machine or deployment server.
Add it to your .gitignore (already included).


## Security Notes

The service_account.json file contains sensitive credentials. Never push it to GitHub.
If accidentally committed, remove it from history and revoke the key in Google Cloud.
For production, consider using environment variables or a secret manager instead of a raw JSON file.

