## YakMan

Basic storage egnostic config manager.

This project is still in a very early alpha stage so if you come across this repo, I would recommend against using it in any production systems.

## Run the project(s)

The local setup is a bit lack luster due to the immaturity of this project as well as Leptos.
Hopefully this will get better with time.


1. Create a `.env` file with the following values
    ```sh
    YAKMAN_ADAPTER=LOCAL_FILE_SYSTEM
    RUST_LOG=info
    YAKMAN_TOKEN_SECRET=12345

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



