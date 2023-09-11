use maud::{html, DOCTYPE};

pub fn render<'a>(client_id: &str, users: impl IntoIterator<Item = (i64, &'a str)>) -> String {
    let fragment = html! {
        (DOCTYPE)
        html {
            head {
                title {"Login to Fakehub"}
            }
            body {
                h1 {
                    "Login"
                }
                ol {
                    @for (user_id, login) in users.into_iter() {
                        li {
                            form action={
                                "/login/oauth/authorize?client_id="
                                (client_id)
                                "user_id="
                                (user_id)
                            } method="post" {
                                input type="submit" { "Login as " (login) }
                            }
                        }
                    }
                }
            }
        }
    };

    fragment.into_string()
}
