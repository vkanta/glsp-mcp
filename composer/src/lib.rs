mod bindings;

use bindings::example::math::math::{self, Point};
use bindings::example::user::user::{self, UserProfile};

struct MathImpl;
struct UserImpl;

impl math::Guest for MathImpl {
    fn distance_from_origin(p: Point) -> f64 {
        (p.x * p.x + p.y * p.y).sqrt()
    }
}

impl user::Guest for UserImpl {
    fn greet_user(profile: UserProfile) -> String {
        format!(
            "Hello, {}! Your ID is {} and you are {}.",
            profile.username,
            profile.id,
            if profile.is_active { "active" } else { "inactive" }
        )
    }
}

