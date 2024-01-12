# YakMan

Basic, storage agnostic config manager.

### ⚠️ This project is still in a very early alpha stage so if you come across this repo, I would recommend against using it in any production systems.

For info on how to use YakMan check out the [docs](./docs/index.md)

## Features

- OAuth support
- Role based access control (RBAC)
- Approval system
- History with easy rollbacks
- Config/Project organization tools


## About this project

My goal with this project is to provide an Open Source storage agnostic config manager for backend systems. The primary audience for this project are software teams that have reached a scale where they need a way to manage application configs across multiple applications/projects.

### Motivations / Goals

- Update application configs quickly without restarting applications
- Language/framework agnostic
- Simple to setup with sane defaults
- Storage engine agnostic, meaning you can use whatever storage system you already have (Blob storage, KV, ect)
- Cost effective
- Avoid vendor lock in
- Kubernetes first (however I would like to support Serverless projects/teams too)

### What this project is not

- A place to store application secrets. 
- Trying to handle Meta/Google/Amazon level scale. (However, this project is also not slow by any means 😉)


## Contributing

### Run the project(s) in dev mode

The local setup is a bit lack luster due to the immaturity of this project.
Hopefully this will get better with time.


First you will need Cargo, Node 20, and PNPM installed:

1. Create a `.env` file with the following values in the root of the project
    ```sh
    YAKMAN_HOST=127.0.0.1
    YAKMAN_PORT=8000
    YAKMAN_ADAPTER=LOCAL_FILE_SYSTEM
    RUST_LOG=info
    
    YAKMAN_ACCESS_TOKEN_SIGNING_KEY=12345
    YAKMAN_REFRESH_TOKEN_ENCRYPTION_KEY='a secret key12345678123456781231'
    YAKMAN_DEFAULT_ADMIN_USER_EMAIL=john.smith@gmail.com

    # Mock OAuth server (docker)
    YAKMAN_OAUTH_TOKEN_URL=http://localhost:4011/connect/token
    YAKMAN_OAUTH_AUTH_URL=http://localhost:4011/connect/authorize
    YAKMAN_OAUTH_REDIRECT_URL=http://localhost:5173/session/oauth-callback
    YAKMAN_OAUTH_ISSUER_URL=http://localhost:4011
    YAKMAN_OAUTH_CLIENT_ID=yakman-mock-client-id
    YAKMAN_OAUTH_CLIENT_SECRET=yakman-mock-client-secret
    YAKMAN_OAUTH_SCOPES=email,profile,openid
    ```
1. Create  a `.env` file with the following values at `./frontend/.env`
   ```sh
   YAKMAN_API_URL=http://127.0.0.1:8000
   ```
1. Run `make install` to install the PNPM dependencies for the frontend
1. Run `make mock-auth` to start a mock OAuth server to login.
1. Run `make fd` to start the frontend server. (where 'fd' stands for 'frontend-dev')
1. Run `make bd` to start the backend server. (where 'bd' stands for 'backend-dev')

