use async_trait::async_trait;
use crate::error::{Error, Result};
use crate::tools::traits::Tool;
use crate::tools::types::{ParameterType, ToolParameter, ToolResult};
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

pub struct WebScraperTool {
    client: Client,
}

impl WebScraperTool {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("RustForge/1.0")
            .build()
            .unwrap();

        Self { client }
    }

    async fn fetch_html(&self, url: &str) -> Result<String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("Failed to fetch URL: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Internal(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let html = response
            .text()
            .await
            .map_err(|e| Error::Internal(format!("Failed to read response: {}", e)))?;

        Ok(html)
    }

    async fn extract_text(&self, url: &str, selector: Option<&str>) -> Result<ToolResult> {
        let html = self.fetch_html(url).await?;
        let document = Html::parse_document(&html);

        let texts = if let Some(sel) = selector {
            let selector = Selector::parse(sel)
                .map_err(|e| Error::Internal(format!("Invalid CSS selector: {:?}", e)))?;

            document
                .select(&selector)
                .map(|el| el.text().collect::<String>())
                .collect::<Vec<_>>()
        } else {
            vec![document.root_element().text().collect::<String>()]
        };

        Ok(ToolResult::success(json!({
            "url": url,
            "texts": texts,
            "count": texts.len()
        })))
    }

    async fn extract_links(&self, url: &str) -> Result<ToolResult> {
        let html = self.fetch_html(url).await?;
        let document = Html::parse_document(&html);

        let selector = Selector::parse("a[href]").unwrap();
        let links: Vec<String> = document
            .select(&selector)
            .filter_map(|el| el.value().attr("href"))
            .map(|href| href.to_string())
            .collect();

        Ok(ToolResult::success(json!({
            "url": url,
            "links": links,
            "count": links.len()
        })))
    }

    async fn extract_attributes(
        &self,
        url: &str,
        selector: &str,
        attribute: &str,
    ) -> Result<ToolResult> {
        let html = self.fetch_html(url).await?;
        let document = Html::parse_document(&html);

        let sel = Selector::parse(selector)
            .map_err(|e| Error::Internal(format!("Invalid CSS selector: {:?}", e)))?;

        let values: Vec<String> = document
            .select(&sel)
            .filter_map(|el| el.value().attr(attribute))
            .map(|val| val.to_string())
            .collect();

        Ok(ToolResult::success(json!({
            "url": url,
            "selector": selector,
            "attribute": attribute,
            "values": values,
            "count": values.len()
        })))
    }

    async fn get_page_info(&self, url: &str) -> Result<ToolResult> {
        let html = self.fetch_html(url).await?;
        let document = Html::parse_document(&html);

        let title = document
            .select(&Selector::parse("title").unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let meta_description = document
            .select(&Selector::parse("meta[name='description']").unwrap())
            .next()
            .and_then(|el| el.value().attr("content"))
            .unwrap_or_default();

        let headings: Vec<String> = document
            .select(&Selector::parse("h1, h2, h3").unwrap())
            .map(|el| el.text().collect::<String>())
            .collect();

        Ok(ToolResult::success(json!({
            "url": url,
            "title": title,
            "description": meta_description,
            "headings": headings
        })))
    }
}

impl Default for WebScraperTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for WebScraperTool {
    fn name(&self) -> &str {
        "web_scraper"
    }

    fn description(&self) -> &str {
        "Fetch and extract data from web pages using CSS selectors"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter::new(
                "operation",
                "Operation: extract_text, extract_links, extract_attributes, page_info",
                ParameterType::String,
                true,
            ),
            ToolParameter::new("url", "URL to scrape", ParameterType::String, true),
            ToolParameter::new(
                "selector",
                "CSS selector (for extract_text, extract_attributes)",
                ParameterType::String,
                false,
            ),
            ToolParameter::new(
                "attribute",
                "Attribute name (for extract_attributes)",
                ParameterType::String,
                false,
            ),
        ]
    }

    async fn execute(&self, params: HashMap<String, Value>) -> Result<ToolResult> {
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing operation parameter".to_string()))?;

        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing url parameter".to_string()))?;

        match operation {
            "extract_text" => {
                let selector = params.get("selector").and_then(|v| v.as_str());
                self.extract_text(url, selector).await
            }
            "extract_links" => self.extract_links(url).await,
            "extract_attributes" => {
                let selector = params
                    .get("selector")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        Error::Internal("Missing selector parameter for extract_attributes".to_string())
                    })?;
                let attribute = params
                    .get("attribute")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        Error::Internal("Missing attribute parameter for extract_attributes".to_string())
                    })?;
                self.extract_attributes(url, selector, attribute).await
            }
            "page_info" => self.get_page_info(url).await,
            _ => Err(Error::Internal(format!("Unknown operation: {}", operation))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = WebScraperTool::new();
        assert_eq!(tool.name(), "web_scraper");
        assert!(!tool.description().is_empty());
        assert_eq!(tool.parameters().len(), 4);
    }

    #[tokio::test]
    async fn test_invalid_operation() {
        let tool = WebScraperTool::new();
        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("invalid"));
        params.insert("url".to_string(), json!("http://example.com"));

        let result = tool.execute(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_url() {
        let tool = WebScraperTool::new();
        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("page_info"));

        let result = tool.execute(params).await;
        assert!(result.is_err());
    }
}
