# YakMan

Basic, storage agnostic config manager.

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

1. Create a `testing-directory/local-files` dir the the project root (git ignored)
1. Create a `.env` file with the following values in the root of the project
    ```sh
    # Log Level
    RUST_LOG=info

    # Token Secrets
    YAKMAN_ACCESS_TOKEN_SIGNING_KEY=12345
    YAKMAN_REFRESH_TOKEN_ENCRYPTION_KEY='a secret key12345678123456781231'

    # Default User
    YAKMAN_DEFAULT_ADMIN_USER_EMAIL=test@null.com
    YAKMAN_DEFAULT_ADMIN_USER_PASSWORD=YakMaster123

    # Adapter
    YAKMAN_ADAPTER=LOCAL_FILE_SYSTEM
    LOCAL_FILE_SYSTEM_DIRECTORY=<path-to-project-directory>/testing-directory/local-files
    ```
1. Create  a `.env` file with the following values at `./frontend/.env`
   ```sh
   YAKMAN_API_URL=http://127.0.0.1:8000
   ```
1. Run `make install` to install the PNPM dependencies for the frontend
1. Run `make fd` to start the frontend server. (where 'fd' stands for 'frontend-dev')
1. Run `make bd` to start the backend server. (where 'bd' stands for 'backend-dev')
