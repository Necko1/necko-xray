# ✋ necko8 AKA necko-xray

![Status: Alpha](https://img.shields.io/badge/Status-Deep%20Alpha-red?style=for-the-badge)
[![Xray Core](https://img.shields.io/badge/Core-Xray-blue?style=for-the-badge)](https://xtls.github.io/en/)
[![Docker](https://img.shields.io/badge/Docker-Ready-2496ED?style=for-the-badge&logo=docker&logoColor=white)](https://hub.docker.com/r/necko1/xray)
![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)

> **A lightweight, containerized VPN (Proxy) management panel powered by Xray-core and Rust.**
>
> *Designed for performance, simplicity, and total control.*

---

## ⚠️ WARNING: EARLY DEVELOPMENT

**This project is currently in DEEP ALPHA.**  
Things may break, configs may change, and features are being built as we speak.  
**NOT RECOMMENDED FOR PRODUCTION** (unless you like living on the edge).

---

## Goals

- [ ] **CLI Management** 
  - [ ] **Client adding/editing** 
- [ ] **TUI Dashboard**
- [ ] **Subscription System** 
- [ ] **Multi-Node Support**
- [ ] **Real-time Statistics (nodes, users etc.)**
- [ ] **Telegram Bot Integration**
- [ ] **Per-IP Limitation**
- [ ] **REST API for management**

\* *This list **WILL** expand over time.*

---

## 📦 Installation

### Method 1: Quick Script (Recommended)

The fastest way to get started. Runs on any Linux server that I know.

#### Download and run the installer
```
curl -sSL https://raw.githubusercontent.com/Necko1/necko-xray/refs/heads/master/install.sh | sudo bash
```

### Method 2: Manual Docker Compose

If you prefer full control:

1.  **Download the docker:**
    ```
    sudo curl -fsSL https://get.docker.com | sh
    ```

2.  **Get into the working directory:**
    ```
    mkdir -p "/opt/necko-xray"
    cd "/opt/necko-xray"
    ```

3.  **Download `docker-compose.yml`:**
    ```
    wget https://raw.githubusercontent.com/Necko1/necko-xray/refs/heads/master/docker-compose.yml
    ```
    
4.  **Create the xray-core.json file:**
    ```
    wget https://raw.githubusercontent.com/Necko1/necko-xray/refs/heads/master/xray-core.json
    ```

5.  **Configure Environment:**
    Create a `.env` file (use `.env.example` as reference).
    ```
    curl -sSL -o .env "https://raw.githubusercontent.com/Necko1/necko-xray/refs/heads/master/.env.example"
    ```
    *Recommendation: **Use the `openssl rand -hex <n>` to generate STRONG passwords***

6.  **Download CLI Wrapper:**
    ```
    wget -O /usr/local/bin/necko-xray https://raw.githubusercontent.com/Necko1/necko-xray/refs/heads/master/necko-xray
    chmod +x /usr/local/bin/necko-xray
    ```

7.  **Start:**
    ```
    docker compose up -d
    ```

---

## 🛠 Architecture

Necko Panel relies on a robust stack:

| Component             | Role                                                                   |
|-----------------------|------------------------------------------------------------------------|
| **xray-core**         | The engine handling Proxy traffic (VLESS Reality, etc.).               |
| **necko-xray (Rust)** | The brain. Manages Xray process, API, and logic.                       |
| **PostgreSQL**        | Long-term storage for users, settings, and history.                    |
| **Valkey (Redis)**    | High-speed cache for real-time traffic stats and online user tracking. |

---

## 🤝 Contributing

Got an idea? Found a bug? 

1.  **Fork** the repository.
2.  **Create a branch** (`git checkout -b feature/NewFeature`).
3.  **Commit your changes** (`git commit -m 'Add some New Feature'`).
4.  **Push to the branch** (`git push origin feature/NewFeature`).
5.  **Open a Pull Request**.

**Ideas & Discussions:**  
Feel free to open an [Issue](https://github.com/Necko1/necko-xray/issues) to discuss new features.

---

## ❤️ Support

If you like where this project is going, give it a ⭐ Star on GitHub!

- **Current Version:** `v1.0.1`  
- **Docker Image:** [`necko1/xray:latest`](https://hub.docker.com/r/necko1/xray)
- **Github repository:** [`Necko1/necko-xray`](https://github.com/Necko1/necko-xray)

---

## License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.
