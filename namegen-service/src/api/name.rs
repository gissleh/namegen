use rocket_contrib::json::Json;
use rocket::State;
use rocket::response::status::{Custom, NoContent, NotFound};
use namegen::{Name};

use crate::store::{Store, NameData, NameHeader};
use rocket::http::Status;

#[get("/")]
pub fn list_names(store: State<Store>) -> Result<Json<Vec<NameHeader>>, Custom<String>> {
    Ok(Json(store.list()))
}

#[get("/<id>")]
pub fn get_name(store: State<Store>, id: String) -> Result<Json<NameData>, NotFound<String>> {
    match store.find(&id) {
        Some(name) => Ok(Json(name)),
        None => Err(NotFound("Not found".to_owned()))
    }
}

#[post("/", format = "json", data = "<input>")]
pub fn create_name(store: State<Store>, input: Json<NameCreateInput>) -> Result<NoContent, Custom<String>> {
    if let Some(_) = store.find(&input.id) {
        return Err(Custom(Status::Conflict, "Name already exists".to_owned()));
    }

    store.save(&input.id, NameData::new(&input.id, input.version, &input.name));

    Ok(NoContent)
}

#[get("/<id>/generate?<amount>&<format>")]
pub fn generate_name(store: State<Store>, id: String, amount: Option<usize>, format: Option<String>) -> Result<Json<Vec<String>>, NotFound<String>> {
    match store.find(&id) {
        Some(name_data) => {
            let name = name_data.name;

            let format = if let Some(format) = format {
                name.formats().find(|f| f.name() == &format).map(|f| f.name())
            } else {
                name.first_format_name()
            };
            if let None = format {
                return Err(NotFound("Format not found".to_owned()))
            }
            let format = format.unwrap();

            let amount = amount.unwrap_or(16);
            let names = name.generate(format).unwrap().take(amount).collect();

            Ok(Json(names))
        }
        None => Err(NotFound("Not found".to_owned()))
    }
}

#[delete("/<id>")]
pub fn delete_name(store: State<Store>, id: String) -> Result<Json<NameData>, NotFound<String>> {
    match store.delete(&id) {
        Some(name) => Ok(Json(name)),
        None => Err(NotFound("Not found".to_owned()))
    }
}

#[derive(Serialize, Deserialize)]
pub struct NameCreateInput {
    pub id: String,
    pub name: Name,
    pub version: u64,
}