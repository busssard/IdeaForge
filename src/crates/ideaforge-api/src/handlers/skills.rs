use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get};
use serde::Serialize;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(get_skills))
}

#[derive(Debug, Serialize)]
pub struct SkillCategory {
    pub category: &'static str,
    pub skills: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct SkillsResponse {
    pub categories: Vec<SkillCategory>,
}

async fn get_skills() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(SkillsResponse {
            categories: vec![
                SkillCategory {
                    category: "Development",
                    skills: vec![
                        "rust",
                        "python",
                        "javascript",
                        "typescript",
                        "go",
                        "java",
                        "c++",
                        "solidity",
                        "react",
                        "vue",
                        "angular",
                        "node.js",
                        "django",
                        "flask",
                        "postgresql",
                        "mongodb",
                        "redis",
                        "docker",
                        "kubernetes",
                        "aws",
                        "gcp",
                        "azure",
                        "devops",
                        "ci/cd",
                        "testing",
                        "security",
                    ],
                },
                SkillCategory {
                    category: "Design",
                    skills: vec![
                        "ui/ux",
                        "graphic-design",
                        "product-design",
                        "figma",
                        "sketch",
                        "adobe-creative-suite",
                        "prototyping",
                        "user-research",
                        "wireframing",
                        "branding",
                        "illustration",
                        "animation",
                        "3d-modeling",
                        "cad",
                    ],
                },
                SkillCategory {
                    category: "Business",
                    skills: vec![
                        "business-strategy",
                        "marketing",
                        "sales",
                        "finance",
                        "accounting",
                        "legal",
                        "fundraising",
                        "investor-relations",
                        "product-management",
                        "project-management",
                        "operations",
                        "hr",
                        "customer-success",
                        "business-development",
                        "partnerships",
                    ],
                },
                SkillCategory {
                    category: "Hardware",
                    skills: vec![
                        "electronics",
                        "pcb-design",
                        "embedded-systems",
                        "iot",
                        "robotics",
                        "mechanical-engineering",
                        "manufacturing",
                        "3d-printing",
                        "prototyping",
                        "firmware",
                        "arduino",
                        "raspberry-pi",
                    ],
                },
                SkillCategory {
                    category: "Creative",
                    skills: vec![
                        "writing",
                        "copywriting",
                        "content-creation",
                        "video-editing",
                        "photography",
                        "music",
                        "audio-production",
                        "storytelling",
                        "blogging",
                        "social-media",
                        "podcasting",
                    ],
                },
                SkillCategory {
                    category: "Domain",
                    skills: vec![
                        "blockchain",
                        "web3",
                        "ai/ml",
                        "data-science",
                        "analytics",
                        "biotech",
                        "healthcare",
                        "fintech",
                        "edtech",
                        "climate-tech",
                        "renewable-energy",
                        "agriculture",
                        "food-tech",
                        "gaming",
                        "vr/ar",
                        "cybersecurity",
                    ],
                },
            ],
        }),
    )
}
