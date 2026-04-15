use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::entities::enums::{IdeaMaturity, IdeaOpenness, UserRole};
use crate::repositories::category_repo::CategoryRepository;
use crate::repositories::idea_repo::IdeaRepository;
use crate::repositories::user_repo::UserRepository;

/// Seed the database with default categories, test users, and sample ideas.
pub async fn seed_database(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    seed_categories(db).await?;
    let users = seed_users(db).await?;
    seed_ideas(db, &users).await?;
    Ok(())
}

struct SeededUser {
    id: Uuid,
}

async fn seed_categories(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let repo = CategoryRepository::new(db);

    let categories = [
        ("Technology", "technology", "Software, hardware, AI, and digital innovation"),
        ("Science", "science", "Research, discoveries, and scientific breakthroughs"),
        ("Art & Design", "art-design", "Visual arts, music, film, and creative expression"),
        ("Social Impact", "social-impact", "Community, sustainability, and social change"),
        ("Business", "business", "Startups, commerce, and entrepreneurship"),
        ("Education", "education", "Learning, teaching, and knowledge sharing"),
        ("Health", "health", "Wellness, medicine, and healthcare innovation"),
        ("Environment", "environment", "Climate, conservation, and green technology"),
    ];

    for (i, (name, slug, desc)) in categories.iter().enumerate() {
        repo.upsert_by_slug(Uuid::new_v4(), name, slug, desc, None, i as i32)
            .await?;
    }

    tracing::info!("Seeded {} categories", categories.len());
    Ok(())
}

async fn seed_users(db: &DatabaseConnection) -> Result<Vec<SeededUser>, sea_orm::DbErr> {
    let repo = UserRepository::new(db);

    let password_hash = ideaforge_auth::password::hash_password("Test1234!")
        .expect("Failed to hash seed password");

    let users_data = [
        ("alice@example.com", "Alice Entrepreneur", UserRole::Entrepreneur),
        ("bob@example.com", "Bob Maker", UserRole::Maker),
    ];

    let mut users = Vec::new();
    for (email, name, role) in &users_data {
        if let Some(existing) = repo.find_by_email(email).await? {
            tracing::debug!("User {email} already exists, skipping");
            users.push(SeededUser { id: existing.id });
            continue;
        }
        let id = Uuid::new_v4();
        repo.create(id, email, &password_hash, name, role.clone()).await?;
        users.push(SeededUser { id });
    }

    tracing::info!("Seeded {} users", users_data.len());
    Ok(users)
}

async fn seed_ideas(db: &DatabaseConnection, users: &[SeededUser]) -> Result<(), sea_orm::DbErr> {
    let repo = IdeaRepository::new(db);
    let cat_repo = CategoryRepository::new(db);

    let tech_cat = cat_repo.find_by_slug("technology").await?;
    let social_cat = cat_repo.find_by_slug("social-impact").await?;
    let edu_cat = cat_repo.find_by_slug("education").await?;

    let ideas: [(usize, &str, &str, &str, IdeaOpenness, Option<Uuid>); 3] = [
        (
            0,
            "Open Source AI Tutor",
            "An AI-powered tutoring platform that adapts to each student's learning style",
            "Build a free, open-source AI tutor that uses spaced repetition and adaptive questioning to help students master any subject. The AI analyzes learning patterns and adjusts difficulty in real-time.",
            IdeaOpenness::Open,
            edu_cat.as_ref().map(|c| c.id),
        ),
        (
            0,
            "Community Tool Library",
            "A platform for neighborhoods to share tools, equipment, and skills",
            "Most power tools sit unused 95% of the time. This platform lets neighbors list tools they're willing to lend, book borrowing slots, and even offer to teach others how to use them safely.",
            IdeaOpenness::Collaborative,
            social_cat.as_ref().map(|c| c.id),
        ),
        (
            1,
            "Rust Game Engine for Education",
            "A simple, well-documented game engine designed for teaching programming",
            "Create a game engine in Rust that prioritizes clear code, extensive documentation, and gentle learning curves. Each module teaches a CS concept: rendering teaches linear algebra, physics teaches calculus, ECS teaches data-oriented design.",
            IdeaOpenness::Open,
            tech_cat.as_ref().map(|c| c.id),
        ),
    ];

    for (user_idx, title, summary, description, openness, cat_id) in &ideas {
        let (existing, _) = repo.list(None, None, None, None, false, None, None, 1, 100).await?;
        if existing.iter().any(|i| i.title == *title) {
            tracing::debug!("Idea '{title}' already exists, skipping");
            continue;
        }
        repo.create(
            Uuid::new_v4(),
            users[*user_idx].id,
            title,
            summary,
            description,
            IdeaMaturity::Spark,
            openness.clone(),
            *cat_id,
        )
        .await?;
    }

    tracing::info!("Seeded {} ideas", ideas.len());
    Ok(())
}
