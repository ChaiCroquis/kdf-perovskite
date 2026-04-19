//! User Segmentation with KDF
//!
//! This example demonstrates how to segment users for targeted marketing:
//! - Core: Mainstream users (standard campaigns)
//! - Edge: Growth potential users (nurture campaigns)
//! - Rare: VIP or at-risk users (personalized attention)
//!
//! Run: cargo run --example kdf_user_segmentation

use kdf::{cosine_similarity, Kdf, Layer};

/// User profile with behavioral features
#[derive(Clone)]
struct UserProfile {
    id: String,
    /// Feature vector: [purchase_freq, avg_order_value, days_since_last,
    ///                  category_diversity, support_tickets, nps_score]
    features: Vec<f64>,
    total_revenue: f64,
    account_age_days: u32,
}

/// Segment with recommended actions
#[derive(Debug)]
struct UserSegment {
    name: String,
    users: Vec<String>,
    recommended_action: String,
    priority: u8,
}

fn segment_users(users: &[UserProfile], threshold: f64) -> Vec<UserSegment> {
    let kdf = Kdf::with_defaults();

    let result = kdf.process(users, threshold, |a, b| {
        cosine_similarity(&a.features, &b.features)
    });

    // Collect users by layer
    let mut core_users = Vec::new();
    let mut edge_users = Vec::new();
    let mut rare_users = Vec::new();

    for (i, user) in users.iter().enumerate() {
        match result.layers[i] {
            Layer::Core => core_users.push(user.id.clone()),
            Layer::Edge => edge_users.push(user.id.clone()),
            Layer::Rare => rare_users.push(user.id.clone()),
        }
    }

    // Further segment Rare users by revenue
    let (vip_users, at_risk_users): (Vec<_>, Vec<_>) = rare_users.iter()
        .map(|id| {
            let user = users.iter().find(|u| &u.id == id).unwrap();
            (id.clone(), user.total_revenue)
        })
        .partition(|(_, revenue)| *revenue > 1000.0);

    vec![
        UserSegment {
            name: "VIP".into(),
            users: vip_users.into_iter().map(|(id, _)| id).collect(),
            recommended_action: "Personal account manager, exclusive offers".into(),
            priority: 1,
        },
        UserSegment {
            name: "At-Risk".into(),
            users: at_risk_users.into_iter().map(|(id, _)| id).collect(),
            recommended_action: "Win-back campaign, satisfaction survey".into(),
            priority: 2,
        },
        UserSegment {
            name: "Growth".into(),
            users: edge_users,
            recommended_action: "Upsell offers, loyalty program invitation".into(),
            priority: 3,
        },
        UserSegment {
            name: "Mainstream".into(),
            users: core_users,
            recommended_action: "Standard newsletter, seasonal promotions".into(),
            priority: 4,
        },
    ]
}

fn main() {
    println!("=== User Segmentation with KDF ===\n");

    // Create user profiles
    let users = vec![
        // Mainstream shoppers (Core cluster)
        UserProfile {
            id: "user_001".into(),
            features: vec![0.5, 0.4, 0.3, 0.5, 0.1, 0.7],
            total_revenue: 500.0,
            account_age_days: 365,
        },
        UserProfile {
            id: "user_002".into(),
            features: vec![0.5, 0.5, 0.3, 0.4, 0.1, 0.7],
            total_revenue: 550.0,
            account_age_days: 400,
        },
        UserProfile {
            id: "user_003".into(),
            features: vec![0.6, 0.4, 0.2, 0.5, 0.2, 0.6],
            total_revenue: 480.0,
            account_age_days: 300,
        },
        UserProfile {
            id: "user_004".into(),
            features: vec![0.5, 0.45, 0.35, 0.45, 0.1, 0.65],
            total_revenue: 520.0,
            account_age_days: 350,
        },

        // Growth potential (Edge)
        UserProfile {
            id: "user_005".into(),
            features: vec![0.7, 0.6, 0.2, 0.6, 0.1, 0.8],
            total_revenue: 800.0,
            account_age_days: 180,
        },
        UserProfile {
            id: "user_006".into(),
            features: vec![0.3, 0.7, 0.4, 0.3, 0.0, 0.9],
            total_revenue: 700.0,
            account_age_days: 90,
        },

        // VIP (Rare - high value)
        UserProfile {
            id: "user_007".into(),
            features: vec![0.95, 0.9, 0.1, 0.9, 0.0, 1.0],
            total_revenue: 5000.0,
            account_age_days: 730,
        },
        UserProfile {
            id: "user_008".into(),
            features: vec![0.9, 0.95, 0.1, 0.85, 0.05, 0.95],
            total_revenue: 4500.0,
            account_age_days: 600,
        },

        // At-risk (Rare - churning)
        UserProfile {
            id: "user_009".into(),
            features: vec![0.1, 0.2, 0.9, 0.2, 0.5, 0.3],
            total_revenue: 200.0,
            account_age_days: 500,
        },
        UserProfile {
            id: "user_010".into(),
            features: vec![0.05, 0.3, 0.95, 0.1, 0.8, 0.2],
            total_revenue: 150.0,
            account_age_days: 400,
        },
    ];

    println!("Total users: {}\n", users.len());

    // =========================================================================
    // 1. Basic KDF Analysis
    // =========================================================================
    println!("--- KDF Layer Analysis ---\n");

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&users, 0.85, |a, b| {
        cosine_similarity(&a.features, &b.features)
    });

    for (i, user) in users.iter().enumerate() {
        let layer = result.layers[i];
        println!("[{:?}] {} - Revenue: ${:.0}, Age: {}d",
            layer, user.id, user.total_revenue, user.account_age_days);
    }
    println!();

    // =========================================================================
    // 2. Segment Users
    // =========================================================================
    println!("--- User Segments ---\n");

    let segments = segment_users(&users, 0.85);

    for segment in &segments {
        println!("📌 {} (Priority: {})", segment.name, segment.priority);
        println!("   Users: {:?}", segment.users);
        println!("   Action: {}", segment.recommended_action);
        println!();
    }

    // =========================================================================
    // 3. Resource Allocation
    // =========================================================================
    println!("--- Resource Allocation ---\n");

    let total_budget = 10000.0;

    for segment in &segments {
        let allocation = match segment.priority {
            1 => 0.35,  // VIP: 35%
            2 => 0.30,  // At-Risk: 30%
            3 => 0.25,  // Growth: 25%
            _ => 0.10,  // Mainstream: 10%
        };
        let budget = total_budget * allocation;
        let per_user = if segment.users.is_empty() {
            0.0
        } else {
            budget / segment.users.len() as f64
        };

        println!("{}: ${:.0} total (${:.0}/user)",
            segment.name, budget, per_user);
    }
    println!();

    // =========================================================================
    // 4. Churn Prediction Insight
    // =========================================================================
    println!("--- Churn Risk Analysis ---\n");

    let at_risk_segment = segments.iter().find(|s| s.name == "At-Risk");
    if let Some(segment) = at_risk_segment {
        println!("⚠️  At-Risk Users Detected: {}", segment.users.len());
        for user_id in &segment.users {
            let user = users.iter().find(|u| &u.id == user_id).unwrap();
            println!("   {} - Last active: {}+ days ago, {} support tickets",
                user_id,
                (user.features[2] * 100.0) as u32,
                (user.features[4] * 10.0) as u32);
        }
        println!("\n   Recommended: Immediate outreach with win-back offer");
    }
    println!();

    // =========================================================================
    // Summary
    // =========================================================================
    println!("=== Summary ===");
    println!("KDF User Segmentation benefits:");
    println!("1. Automatic discovery of user clusters");
    println!("2. Identify VIP users (Rare + high revenue)");
    println!("3. Detect at-risk users (Rare + low engagement)");
    println!("4. Optimize marketing spend allocation");
    println!("5. No manual threshold tuning for segments");
}
