use bigdecimal::{BigDecimal, FromPrimitive};
use super::*;
use serde_json::json;

#[get("/car/<uid>")]
pub async fn car_show(conn: LibraryDbConn, uid: String, user: StaffEntity) -> Result<Template> {
    use schema::car::dsl::*;
    let data: CarEntity = conn
        .run(move |c| car.filter(plate_number.eq(uid)).first(c))
        .await?;
    Ok(Template::render(
        "car/show",
        json!({"data": data, "user": user}),
    ))
}

#[post("/car", data = "<new>")]
pub async fn car_new(
    conn: LibraryDbConn,
    new: Form<CarEntityForm>,
    _user: StaffEntity,
) -> Result<Redirect> {
    use schema::car::dsl::*;
    let conv_bigdecimal: BigDecimal = BigDecimal::from_f32(new.price_per_day).expect("Conversion failed").with_scale(2);
    let converted = CarEntity {
        plate_number: new.plate_number.clone(),
        car_model_id: new.car_model_id,
        available: new.available,
        condition: new.condition.clone(),
        price_per_day: conv_bigdecimal
    };
    conn.run(move |c| insert_into(car).values(converted).execute(c))
        .await?;
    Ok(Redirect::to(uri!(car_list)))
}

#[put("/car/<uid>", data = "<updated>")]
pub async fn car_update(
    conn: LibraryDbConn,
    uid: String,
    updated: Form<CarForm>,
    _user: StaffEntity,
) -> Result<Redirect> {
    use schema::car::dsl::*;
    let target = update(car).filter(plate_number.eq(uid));
    let conv_bigdecimal: BigDecimal = BigDecimal::from_f32(updated.price_per_day).expect("Conversion failed").with_scale(2);
    let converted = Car {
        car_model_id: updated.car_model_id,
        available: updated.available,
        condition: updated.condition.clone(),
        price_per_day: conv_bigdecimal
    };
    conn.run(move |c| target.set(converted).execute(c)).await?;
    Ok(Redirect::to(uri!(car_list)))
}

#[delete("/car/<uid>")]
pub async fn car_delete(conn: LibraryDbConn, uid: String, _user: StaffEntity) -> Result<Redirect> {
    use schema::car::dsl::*;
    conn.run(move |c| delete(car).filter(plate_number.eq(uid)).execute(c))
        .await?;

    Ok(Redirect::to(uri!(car_list)))
}

#[get("/car/add")]
pub async fn car_add_menu(conn: LibraryDbConn, user: StaffEntity) -> Result<Template> {
    use schema::car_model::dsl::*;
    let car_models = conn
        .run(|c| car_model.load::<CarModelEntity>(c))
        .await?;
    Ok(Template::render(
        "car/add",
        json!({"car_models": car_models, "user": user}),
    ))
}

#[get("/car/update/<uid>")]
pub async fn car_update_menu(
    conn: LibraryDbConn,
    uid: String,
    user: StaffEntity,
) -> Result<Template> {
    use schema::car::dsl::*;
    let data: CarEntity = conn
        .run(move |c| car.filter(plate_number.eq(uid)).first(c))
        .await?;
    use schema::car_model::dsl::*;
    let car_models = conn
        .run(|c| car_model.load::<CarModelEntity>(c))
        .await?;
    Ok(Template::render(
        "car/update",
        json!({"data": data,
            "car_models": car_models,
            "user": user
        }),
    ))
}

#[get("/car/list")]
pub async fn car_list(conn: LibraryDbConn, user: StaffEntity) -> Result<Template> {
    use schema::car::dsl::*;
    let all = conn.run(|c| car.load::<CarEntity>(c)).await?;
    let context = json!({
        "entities": all,
        "user" : user
    });
    Ok(Template::render(
        "car/list",
        json!({"data": context, "user": user}),
    ))
}