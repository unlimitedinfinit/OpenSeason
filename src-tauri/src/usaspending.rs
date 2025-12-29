use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct AwardSummary {
    pub generated_internal_id: String,
    pub date_signed: String,
    pub description: Option<String>,
    pub total_obligation: f64,
    pub awarding_agency: String,
    pub recipient_name: String,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    results: Vec<ApiResult>,
}

#[derive(Deserialize, Debug)]
struct ApiResult {
    #[serde(default)]
    generated_internal_id: String,
    #[serde(default)]
    Date_Signed: String, 
    #[serde(default)]
    Description: String,
    #[serde(default)]
    Award_Amount: f64,
    #[serde(default)]
    Awarding_Agency: String,
    #[serde(default)]
    Recipient_Name: String,
}

pub fn check_target(target_name: &str) -> Result<Vec<AwardSummary>, String> {
    let client = Client::new();
    let url = "https://api.usaspending.gov/api/v2/search/spending_by_award/";

    // Simplified filter for awards > $100k related to keyword
    let payload = json!({
        "filters": {
            "keywords": [target_name],
            "award_amounts": [
                { "lower_bound": 100000.0 }
            ],
            "award_type_codes": ["A", "B", "C", "D"] // Contracts
        },
        "fields": [
            "Award ID", 
            "Recipient Name", 
            "Start Date", 
            "End Date", 
            "Award Amount", 
            "Description", 
            "Awarding Agency", 
            "Generated Internal ID",
            "Date Signed"
        ],
        "limit": 10,
        "page": 1
    });

    let res = client.post(url)
        .json(&payload)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("API Error: {}", res.status()));
    }

    let api_response: ApiResponse = res.json().map_err(|e| format!("Parse error: {}", e))?;

    // Map to clean struct
    let summaries = api_response.results.into_iter().map(|r| AwardSummary {
        generated_internal_id: r.generated_internal_id,
        date_signed: r.Date_Signed,
        description: Some(r.Description),
        total_obligation: r.Award_Amount,
        awarding_agency: r.Awarding_Agency,
        recipient_name: r.Recipient_Name,
    }).collect();

    Ok(summaries)
}
