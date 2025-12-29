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
    #[serde(rename = "generated_internal_id", default)]
    generated_internal_id: String,

    #[serde(rename = "action_date", default)]
    date_signed: String, 
    
    #[serde(rename = "description", default)]
    description: String,
    
    // Contracts & Grants
    #[serde(rename = "total_obligation", default)]
    total_obligation: Option<f64>,

    // Loans
    #[serde(rename = "face_value_loan", default)]
    face_value_loan: Option<f64>,
    
    // Fallback for some loan types
    #[serde(rename = "original_loan_subsidy_cost", default)]
    original_loan_subsidy_cost: Option<f64>,

    #[serde(rename = "awarding_agency_name", default)]
    awarding_agency: String,

    #[serde(rename = "recipient_name", default)]
    recipient_name: String,
}

pub fn check_target(target_name: &str) -> Result<Vec<AwardSummary>, String> {
    let client = Client::new();
    let base_url = "https://api.usaspending.gov/api/v2/search/spending_by_award/";

    // Group 1: Contracts
    // Codes: A, B, C, D
    let contracts_payload = make_payload(target_name, vec!["A", "B", "C", "D"], vec![
        "Award ID", "Recipient Name", "Total Obligation", "Awarding Agency", "Description", "Action Date"
    ]);

    // Group 2: Assistance (Loans, Grants, Direct Payments)
    // Codes: 02, 03, 04, 05 (Grants), 06, 10 (Direct Payments), 07, 08 (Loans), 09, 11 (Insurance/Other)
    let assistance_payload = make_payload(target_name, vec![
        "02", "03", "04", "05", "06", "07", "08", "09", "10", "11"
    ], vec![
        "Award ID", "Recipient Name", "Total Obligation", "Face Value of Loan", "Original Loan Subsidy Cost", 
        "Awarding Agency", "Description", "Action Date"
    ]);

    let mut all_results = Vec::new();

    // Fetch Contracts
    if let Ok(results) = fetch_group(&client, base_url, &contracts_payload) {
        all_results.extend(results);
    }

    // Fetch Assistance
    if let Ok(results) = fetch_group(&client, base_url, &assistance_payload) {
        all_results.extend(results);
    }
    
    // Map to Summary
    let summaries = all_results.into_iter().map(|r| {
        // Determine value priority: Face Value -> Total Obligation -> Subsidy -> 0.0
        let value = r.face_value_loan
            .or(r.total_obligation)
            .or(r.original_loan_subsidy_cost)
            .unwrap_or(0.0);

        AwardSummary {
            generated_internal_id: r.generated_internal_id,
            date_signed: r.date_signed,
            description: Some(r.description),
            total_obligation: value,
            awarding_agency: r.awarding_agency,
            recipient_name: r.recipient_name,
        }
    }).collect::<Vec<_>>(); // Collect to vec to allow printing

    println!("Final merged results for '{}': {:?}", target_name, summaries);

    Ok(summaries)
}

fn make_payload(keyword: &str, codes: Vec<&str>, fields: Vec<&str>) -> serde_json::Value {
    json!({
        "filters": {
            "keywords": [keyword],
            "time_period": [
                {"start_date": "2010-01-01", "end_date": "2025-12-31"}
            ],
            "award_type_codes": codes
        },
        "fields": fields,
        "limit": 10,
        "page": 1
    })
}

fn fetch_group(client: &Client, url: &str, payload: &serde_json::Value) -> Result<Vec<ApiResult>, String> {
    let res = client.post(url)
        .json(payload)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("API Error: {}", res.status()));
    }

    let text_response = res.text().map_err(|e| format!("Read error: {}", e))?;
    
    let api_response: ApiResponse = serde_json::from_str(&text_response)
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(api_response.results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_feeding_our_future() {
        let result = check_target("Feeding Our Future");
        match result {
            Ok(summaries) => {
                println!("Found {} records", summaries.len());
                for s in summaries {
                    println!(" - {} (${}): {}", s.recipient_name, s.total_obligation, s.description.unwrap_or_default());
                }
            },
            Err(e) => println!("Error: {}", e),
        }
    }
}
