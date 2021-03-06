#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
use diesel::{delete, insert_into, prelude::*, update};
use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::http::{Cookie, CookieJar};
use rocket::request::FlashMessage;
use rocket::response::{Debug, Flash, Redirect};
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::path::PathBuf;

mod models;
mod schema;
use models::*;
mod auth;
use auth::*;
mod car;
mod car_model;
mod customer;
mod manufacturer;
mod rental_cases;

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

#[get("/data/<uid>")]
async fn data(conn: LibraryDbConn, uid: i32) -> Result<Template> {
    use schema::staff::dsl::*;
    let data: StaffEntity = conn
        .run(move |c| staff.filter(staff_id.eq(uid)).first(c))
        .await?;
    Ok(Template::render("show", data))
}

#[post("/data", data = "<new_staff>")]
async fn new_data(conn: LibraryDbConn, new_staff: Form<Staff>) -> Result<Redirect> {
    use schema::staff::dsl::*;
    conn.run(move |c| insert_into(staff).values(&*new_staff).execute(c))
        .await?;
    Ok(Redirect::to(uri!(index)))
}

#[put("/data/<uid>", data = "<updated_user>")]
async fn update_data(conn: LibraryDbConn, uid: i32, updated_user: Form<Staff>) -> Result<Redirect> {
    use schema::staff::dsl::*;
    let target = update(staff).filter(staff_id.eq(uid));
    conn.run(move |c| target.set(&*updated_user).execute(c))
        .await?;
    Ok(Redirect::to(uri!(index)))
}

#[delete("/data/<uid>")]
async fn delete_data(conn: LibraryDbConn, uid: i32) -> Result<Redirect> {
    use schema::staff::dsl::*;
    conn.run(move |c| delete(staff).filter(staff_id.eq(uid)).execute(c))
        .await?;

    Ok(Redirect::to(uri!(index)))
}

#[get("/register")]
fn add_staff() -> Template {
    Template::render("add", HashMap::<i32, i32>::new())
}

#[get("/data/update/<uid>")]
async fn update_staff(conn: LibraryDbConn, uid: i32) -> Result<Template> {
    use schema::staff::dsl::*;
    let data: StaffEntity = conn
        .run(move |c| staff.filter(staff_id.eq(uid)).first(c))
        .await?;
    Ok(Template::render("update", data))
}

#[get("/")]
async fn index(_conn: LibraryDbConn, user: Option<StaffEntity>) -> Redirect {
    match user {
        Option::Some(_) => Redirect::to(uri!("/car/list")),
        Option::None => Redirect::to(uri!("/login"))
    }
}

#[get("/login")]
fn login(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render("login", &flash)
}

#[post("/login", data = "<login>")]
async fn post_login(
    conn: LibraryDbConn,
    jar: &CookieJar<'_>,
    login: Form<Login<'_>>,
) -> Result<Redirect, Flash<Redirect>> {
    use schema::staff::dsl::*;
    let email_clone = login.email.to_string();
    let staff_password = conn
        .run(|c| {
            staff
                .select(password)
                .filter(email.eq(email_clone))
                .get_result::<String>(c)
        })
        .await;
    match staff_password {
        Ok(pwd) => {
            if pwd == login.password {
                jar.add_private(Cookie::new("user_email", login.email.to_string()));
                jar.add_private(Cookie::new("user_password", pwd));
                Ok(Redirect::to(uri!(index)))
            } else {
                Err(Flash::error(Redirect::to(uri!(login)), "Wrong password"))
            }
        }
        Err(_) => Err(Flash::error(Redirect::to(uri!(login)), "Email not found.")),
    }
}

#[get("/logout")]
async fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove_private(Cookie::named("user_email"));
    jar.remove_private(Cookie::named("user_password"));
    Redirect::to(uri!("/"))
}

#[get("/public/<file..>")]
async fn public_file(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(&format!("public/{}", file.to_str()?))
        .await
        .ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                login,
                post_login,
                logout,
                public_file,
                data,
                new_data,
                update_data,
                delete_data,
                add_staff,
                update_staff,
                manufacturer::man_list,
                manufacturer::man_add_menu,
                manufacturer::man_update_menu,
                manufacturer::man_show,
                manufacturer::man_new,
                manufacturer::man_update,
                manufacturer::man_delete,
                car_model::car_mod_list,
                car_model::car_mod_add_menu,
                car_model::car_mod_update_menu,
                car_model::car_mod_show,
                car_model::car_mod_new,
                car_model::car_mod_update,
                car_model::car_mod_delete,
                car::car_list,
                car::car_add_menu,
                car::car_update_menu,
                car::car_show,
                car::car_new,
                car::car_update,
                car::car_delete,
                car::car_diagram_info,
                customer::customer_list,
                customer::customer_rental_cases,
                customer::customer_add_menu,
                customer::customer_update_menu,
                customer::customer_show,
                customer::customer_new,
                customer::customer_update,
                customer::customer_delete,
                rental_cases::rental_cases_list,
                rental_cases::rental_cases_add_menu,
                rental_cases::rental_cases_update_menu,
                rental_cases::rental_cases_show,
                rental_cases::rental_cases_new,
                rental_cases::rental_cases_update,
                rental_cases::rental_cases_delete,
                rental_cases::rental_cases_excel,
                rental_cases::rental_cases_upload_excel
            ],
        )
        .attach(Template::fairing())
        .attach(LibraryDbConn::fairing())
}
