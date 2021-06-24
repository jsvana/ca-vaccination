use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
struct Name {
    family: String,
    given: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct VaccineCoding {
    system: String,
    code: String,
}

#[derive(Debug, Deserialize)]
struct VaccineCode {
    coding: Vec<VaccineCoding>,
}

#[derive(Debug, Deserialize)]
struct Patient {
    reference: String,
}

#[derive(Debug, Deserialize)]
struct Actor {
    display: String,
}

#[derive(Debug, Deserialize)]
struct Performer {
    actor: Actor,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "resourceType", rename_all = "camelCase")]
enum Resource {
    #[serde(rename = "Patient")]
    Patient {
        name: Vec<Name>,
        #[serde(rename = "birthDate")]
        birth_date: String,
    },
    #[serde(rename = "Immunization")]
    Immunization {
        #[serde(rename = "lotNumber")]
        lot_number: String,
        status: String,
        #[serde(rename = "vaccineCode")]
        vaccine_code: VaccineCode,
        patient: Patient,
        #[serde(rename = "occurrenceDateTime")]
        occurrence_date_time: String,
        performer: Vec<Performer>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    full_url: String,
    resource: Resource,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FhirBundle {
    resource_type: String,
    r#type: String,
    entry: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CredentialSubject {
    fhir_version: String,
    fhir_bundle: FhirBundle,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Vc {
    r#type: Vec<String>,
    credential_subject: CredentialSubject,
}

#[derive(Debug, Deserialize)]
struct Body {
    iss: String,
    nbf: i32,
    vc: Vc,
}

fn main() {
    let img = image::open("/home/jsvana/Downloads/ca_qr.png").unwrap();

    // convert to gray scale
    let img_gray = img.into_luma8();

    // create a decoder
    let mut decoder = quircs::Quirc::default();

    // identify all qr codes
    let codes = decoder.identify(
        img_gray.width() as usize,
        img_gray.height() as usize,
        &img_gray,
    );

    for code in codes {
        let code = code.expect("failed to extract qr code");
        let decoded = code.decode().expect("failed to decode qr code");
        let code = std::str::from_utf8(&decoded.payload).unwrap();

        let parts: Vec<&str> = code.split('/').collect();

        let digits = parts.get(1).expect("malformed token");

        let mut letters: Vec<char> = Vec::new();

        for chunk in digits.chars().collect::<Vec<char>>().chunks(2) {
            let chunk: String = chunk.iter().collect();
            let chunk_as_int = chunk.parse::<u32>().expect("failed to parse int");

            letters.push(std::char::from_u32(chunk_as_int + 45).expect("unknown char"));
        }

        let output: String = letters.into_iter().collect();

        let parts: Vec<&str> = output.split('.').collect();
        let contents = base64::decode_config(parts[1], base64::URL_SAFE_NO_PAD)
            .expect("failed to decode base64");
        let decoded = inflate::inflate_bytes(&contents).unwrap();
        let decoded = std::str::from_utf8(&decoded).unwrap();

        let data: Body = serde_json::from_str(&decoded).expect("failed to deserialize");

        for entry in data.vc.credential_subject.fhir_bundle.entry {
            match entry.resource {
                Resource::Patient { name, .. } => {
                    let name = &name[0];
                    println!("Patient: {} {}", name.given.join(" "), name.family);
                }
                Resource::Immunization {
                    lot_number,
                    status,
                    occurrence_date_time,
                    performer,
                    ..
                } => {
                    println!(
                        "Immunization: {}, {} by {} on {}",
                        lot_number, status, performer[0].actor.display, occurrence_date_time
                    );
                }
            }
        }
    }
}
