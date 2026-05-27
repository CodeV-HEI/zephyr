use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::process::{Command, Stdio};
use std::io::Write;
use std::str;

#[derive(Serialize, Deserialize)]
pub struct CssResult {
    pub css: String,
}

#[wasm_bindgen]
pub fn compile_tailwind_classes(classes: &str) -> Result<JsValue, JsError> {
    // Lancer tailwindcss en mode stdin
    let mut child = Command::new("tailwindcss")
        .arg("--stdin")
        .arg("--minify")
        .current_dir("/tmp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| JsError::new(&format!("Failed to spawn tailwindcss: {}", e)))?;

    // Écrire les classes dans stdin
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    stdin
        .write_all(classes.as_bytes())
        .map_err(|e| JsError::new(&format!("Failed to write to stdin: {}", e)))?;
    stdin
        .flush()
        .map_err(|e| JsError::new(&format!("Failed to flush stdin: {}", e)))?;
    // Fermer stdin pour signaler la fin
    drop(stdin);

    // Récupérer la sortie
    let output = child
        .wait_with_output()
        .map_err(|e| JsError::new(&format!("Failed to read output: {}", e)))?;

    if !output.status.success() {
        let err = str::from_utf8(&output.stderr).unwrap_or("unknown error");
        return Err(JsError::new(&format!("Tailwind error: {}", err)));
    }

    let css = str::from_utf8(&output.stdout).map_err(|e| JsError::new(&e.to_string()))?;
    let result = CssResult { css: css.to_string() };
    let json = serde_json::to_string(&result).unwrap();
    Ok(JsValue::from_str(&json))
}