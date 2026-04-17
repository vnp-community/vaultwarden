pub mod tenants;

use rocket::Route;

pub fn routes() -> Vec<Route> {
    tenants::routes()
}
