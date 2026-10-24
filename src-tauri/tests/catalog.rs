use squat_souls_lib::app::*;

#[test]
fn catalog_is_read_from_the_latest_release_in_the_language() {
    assert_eq!(
        catalog_url(&release_catalogs("https://github.com/me/app"), Language::Fr),
        "https://github.com/me/app/releases/latest/download/catalog.fr.json"
    );
}

#[test]
fn catalog_is_read_from_an_address_ending_with_a_slash() {
    assert_eq!(
        catalog_url("http://localhost:1420/", Language::En),
        "http://localhost:1420/catalog.en.json"
    );
}

#[test]
fn catalog_is_a_list_of_share_texts() {
    assert_eq!(
        read_catalog(r#"["squatsouls/game/a:xyz","squatsouls/game/b:abc"]"#).unwrap(),
        ["squatsouls/game/a:xyz", "squatsouls/game/b:abc"]
    );
    for body in ["", "<html>", r#"{"games":[]}"#, "[1]"] {
        assert_eq!(
            read_catalog(body).unwrap_err().code,
            "catalog_invalid",
            "{body}"
        );
    }
}
