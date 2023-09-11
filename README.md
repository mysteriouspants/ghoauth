# ghoauth

Just enough Github client to drive an OAuth workflow.

```rust
use ghoauth::GithubClient;

let github_client = GithubClient::new(CLIENT_ID, CLIENT_SECRET);

// send the user to the authorization url and wait for them to be
// redirected back to your application with a login code
let code = send_user_to_authz_url(client.authorization_url()).await?;
// exchange that code for an api token
let token = client.get_access_token(&code).await?;
// use that token to query the api about the user
let user_detail = github_client.get_user_detail(
    &token.access_token
).await?;

println!("user: {}", user_detail.login);
```

This crate also includes a "Fakehub," which is a mock version of Github with
just enough implemented to serve as a stubbed-out authentication endpoint. It
is designed with automated testing in mind.

```rust
use ghoauth::fakehub::{Fakehub, User};

let fakehub = Fakehub::new()?;

// configure fakehub with a stored client, and return a GithubClient
// configured as that client.
let github_client = fakehub.add_client(CLIENT_ID, CLIENT_SECRET).await?;

// add as many fake users to fakehub, maybe one for each role in your
// test suite, eg. a normal user and an administrator.
fakehub.add_user(USER_ID, User {
    login: USER.to_string(),
    avatar_url: USER_AVATAR_URL.to_string(),
    html_url: USER_HTML_URL.to_string(),
}).await;

// simulate that user going to the authorization url and logging in.
let code = fakehub.get_code().await?;

// the ordinary oauth flow continues unchanged.
let token = github_client.get_access_token(&code).await?;
let user_detail = github_client.get_user_detail(&token.access_token).await?;

assert_eq!(USER, user_detail.login);

// if re-using a process in many tests, remember to shut down the
// fakehub to release ports.
fakehub.shutdown().await;
```

## License

I want you to be able to use this software regardless of who you may be, what
you are working on, or the environment in which you are working on it - I hope
you'll use it for good and not evil! To this end, ghoauth is licensed under
the [2-clause BSD][2cbsd] license, with other licenses available by request.
Happy coding!

[2cbsd]: https://opensource.org/licenses/BSD-2-Clause
