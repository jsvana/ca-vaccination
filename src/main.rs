use std::char::from_u32;
use std::env;
use std::str::from_utf8;

use anyhow::{anyhow, Context, Result};
use base64::{decode_config, URL_SAFE_NO_PAD};
use inflate::inflate_bytes;
use quircs::{Code, Quirc};
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

impl Body {
    fn print(&self) {
        for entry in &self.vc.credential_subject.fhir_bundle.entry {
            match &entry.resource {
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
                        lot_number.trim(),
                        status,
                        performer[0].actor.display,
                        occurrence_date_time
                    );
                }
            }
        }
    }
}

fn decode(code: Code) -> Result<Body> {
    let decoded = code.decode().context("failed to decode qr code")?;
    let code = from_utf8(&decoded.payload).context("failed to parse UTF-8")?;

    let parts: Vec<&str> = code.split('/').collect();

    let digits = parts.get(1).ok_or_else(|| anyhow!("malformed token"))?;

    let mut letters: Vec<char> = Vec::new();

    for chunk in digits.chars().collect::<Vec<char>>().chunks(2) {
        let chunk: String = chunk.iter().collect();
        let chunk_as_int = chunk.parse::<u32>().context("failed to parse int")?;

        letters.push(from_u32(chunk_as_int + 45).context("unknown char")?);
    }

    let output: String = letters.into_iter().collect();

    let parts: Vec<&str> = output.split('.').collect();
    let contents = decode_config(parts[1], URL_SAFE_NO_PAD).context("failed to decode base64")?;
    let decoded = inflate_bytes(&contents)
        .map_err(|e| anyhow!("failed to inflate compressed data: {}", e))?;
    let decoded = from_utf8(&decoded).context("failed to parse UTF-8")?;

    serde_json::from_str(&decoded).context("failed to deserialize")
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let path = args
        .get(1)
        .ok_or_else(|| anyhow!("Usage: cargo run <qr_code_path>"))?;
    let img = image::open(path)?;

    let img_gray = img.into_luma8();
    let mut decoder = Quirc::default();
    let codes = decoder.identify(
        img_gray.width() as usize,
        img_gray.height() as usize,
        &img_gray,
    );

    for code in codes {
        let code = code.context("failed to extract qr code")?;
        let body = decode(code)?;
        body.print();
    }

    Ok(())
}
