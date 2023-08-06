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
- Storage engine agnostic, meaning you can use whatever storage system you already have (Blob storage, SQL, KV, ect)
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


First you will need Cargo and Node 18 installed:

1. Create a `.env` file with the following values in the root of the project
    ```sh
    YAKMAN_ADAPTER=LOCAL_FILE_SYSTEM
    RUST_LOG=info
    YAKMAN_TOKEN_SECRET=12345
    YAKMAN_DEFAULT_ADMIN_USER_EMAIL=<your-email>

    # Google OAuth
    YAKMAN_OAUTH_PROVIDER=GOOGLE
    YAKMAN_OAUTH_REDIRECT_URL=http://127.0.0.1:8080/oauth-callback
    YAKMAN_OAUTH_TOKEN_URL=https://www.googleapis.com/oauth2/v3/token
    YAKMAN_OAUTH_AUTH_URL=https://accounts.google.com/o/oauth2/v2/auth?prompt=consent&access_type=offline
    YAKMAN_OAUTH_SCOPES=email,profile,openid
    # Be sure to add values for these
    YAKMAN_OAUTH_CLIENT_ID=
    YAKMAN_OAUTH_CLIENT_SECRET=
    ```
1. Create  a `.env` file with the following values at `./frontend/.env`
   ```sh
   YAKMAN_API_URL=http://127.0.0.1:8000
   ```
1. Run `make install` to install the NPM dependencies for the frontend
1. Run `make fd` to start the frontend server. (where 'fd' stands for 'frontend-dev')
1. Run `make bd` to start the backend server. (where 'bd' stands for 'backend-dev')

### Run the project(s) in docker (release mode)

Of course to do this you will need Docker installed

1. Setup the `.env` file in step 1 of the dev mode section above
   - Note: The frontend application will run on port 3000, so you will want to change the port in the oauth redirect env var.  
1. Run `make build-backend` to build the backend docker image
1. Run `make build-frontend` to build the frontend docker image
1. Run `docker compose up` to start the application 


