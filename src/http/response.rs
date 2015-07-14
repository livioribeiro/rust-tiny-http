pub struct Response {
    body: String,
}

impl Response {
    pub fn new(body: &str) -> Response {
        Response {
            body: body.to_string()
        }
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}
