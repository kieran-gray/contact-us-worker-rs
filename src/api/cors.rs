use worker::Response;

pub struct CorsHeaders {
    allowed_origins: Vec<String>,
}

impl CorsHeaders {
    pub fn new(allowed_origins: Vec<String>) -> Self {
        Self { allowed_origins }
    }

    pub fn is_allowed(&self, origin: &Option<String>) -> bool {
        if self.allowed_origins.is_empty() {
            return true;
        }
        origin
            .as_ref()
            .is_some_and(|o| self.allowed_origins.contains(o))
    }

    pub fn add_to_response(
        &self,
        mut response: Response,
        origin: Option<String>,
    ) -> worker::Result<Response> {
        if let Some(origin_value) = origin
            && (self.allowed_origins.is_empty() || self.allowed_origins.contains(&origin_value))
        {
            response
                .headers_mut()
                .set("Access-Control-Allow-Origin", &origin_value)?;
            response
                .headers_mut()
                .set("Access-Control-Allow-Methods", "POST, OPTIONS")?;
            response
                .headers_mut()
                .set("Access-Control-Allow-Headers", "Content-Type")?;
        }
        Ok(response)
    }

    pub fn preflight_response(&self, origin: Option<String>) -> worker::Result<Response> {
        let response = Response::empty()?;
        let mut response = self.add_to_response(response, origin)?;

        response
            .headers_mut()
            .set("Access-Control-Max-Age", "86400")?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_allowed_empty_allowed_list() {
        let cors = CorsHeaders {
            allowed_origins: vec![],
        };

        assert!(cors.is_allowed(&Some("http://example.com".to_string())));
        assert!(cors.is_allowed(&Some("http://evil.com".to_string())));
        assert!(cors.is_allowed(&None));
    }

    #[test]
    fn test_is_allowed_with_allowed_list() {
        let cors = CorsHeaders {
            allowed_origins: vec![
                "http://localhost:5173".to_string(),
                "https://example.com".to_string(),
            ],
        };

        assert!(cors.is_allowed(&Some("http://localhost:5173".to_string())));
        assert!(cors.is_allowed(&Some("https://example.com".to_string())));

        assert!(!cors.is_allowed(&Some("http://evil.com".to_string())));
        assert!(!cors.is_allowed(&Some("https://different.com".to_string())));

        assert!(!cors.is_allowed(&None));
    }

    #[test]
    fn test_is_allowed_exact_match() {
        let cors = CorsHeaders {
            allowed_origins: vec!["https://example.com".to_string()],
        };

        assert!(cors.is_allowed(&Some("https://example.com".to_string())));
        assert!(!cors.is_allowed(&Some("http://example.com".to_string())));
        assert!(!cors.is_allowed(&Some("https://sub.example.com".to_string())));
    }
}
