# YakMan

Basic storage egnostic config manager.

### ⚠️ This project is still in a very early alpha stage so if you come across this repo, I would recommend against using it in any production systems.

## Features

- OAuth support
- Role based access control (RBAC)
- Approval system
- History with easy rollbacks
- Config/Project organization tools


## About this project

My goal with this project is to provide an Open Source storage egnostic config manager for backend systems. The primary audience for this project are software teams that have reached a scale where they need a way to manage application configs across multiple applications/projects.

### Motivations / Goals

- Update application configs quickly without restarting applications
- Language/framework egonistic
- Simple to setup with sane defaults
- Storage engine egonistic, meaning you can use whatever storage system you already have (Blob storage, SQL, KV, ect)
- Low cost effective
- Avoid vendor lock in
- Kubernetes first (however I would like to support Serverless projects/teams too)

### What this project is not

- A place to store application secrets. 
- Trying to handle Meta/Google/Amazon level scale. (However, this project is also not slow by any means 😉)


## Contributing

### Run the project(s)

The local setup is a bit lack luster due to the immaturity of this project as well as Leptos.
Hopefully this will get better with time.


1. Create a `.env` file with the following values
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
2. Run `npm install` from the `frontend` directory
3. Run `cargo run` to start the backend server
4. Run `make watch-tailwind` and `make leptos` in seperate terminals.



