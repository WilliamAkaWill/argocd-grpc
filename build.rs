use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Dem Compiler sagen, dass 'rust_analyzer' eine gültige Konfiguration ist
    println!("cargo:rustc-check-cfg=cfg(rust_analyzer)");

    // Zielverzeichnis für alle Proto-Includepfade im target-Ordner ermitteln
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let proto_include = out_dir.join("proto_include");
    
    fs::create_dir_all(&proto_include)
        .map_err(|e| format!("Fehler beim Erstellen von {:?}: {}", proto_include, e))?;

    // 1. Alle benötigten K8s- und Google-Protos
    let base_urls = [
        // Google APIs
        ("https://raw.githubusercontent.com/googleapis/googleapis/master/google/api/annotations.proto", "google/api/annotations.proto"),
        ("https://raw.githubusercontent.com/googleapis/googleapis/master/google/api/http.proto", "google/api/http.proto"),
        
        // Gogo Protobuf
        ("https://raw.githubusercontent.com/gogo/protobuf/master/gogoproto/gogo.proto", "gogoproto/gogo.proto"),
        
        // K8s Apimachinery, API & Core
        ("https://raw.githubusercontent.com/kubernetes/apimachinery/master/pkg/apis/meta/v1/generated.proto", "k8s.io/apimachinery/pkg/apis/meta/v1/generated.proto"),
        ("https://raw.githubusercontent.com/kubernetes/apimachinery/master/pkg/runtime/generated.proto", "k8s.io/apimachinery/pkg/runtime/generated.proto"),
        ("https://raw.githubusercontent.com/kubernetes/apimachinery/master/pkg/runtime/schema/generated.proto", "k8s.io/apimachinery/pkg/runtime/schema/generated.proto"),
        ("https://raw.githubusercontent.com/kubernetes/apimachinery/master/pkg/util/intstr/generated.proto", "k8s.io/apimachinery/pkg/util/intstr/generated.proto"),
        ("https://raw.githubusercontent.com/kubernetes/apimachinery/master/pkg/api/resource/generated.proto", "k8s.io/apimachinery/pkg/api/resource/generated.proto"),
        ("https://raw.githubusercontent.com/kubernetes/api/master/core/v1/generated.proto", "k8s.io/api/core/v1/generated.proto"),
        ("https://raw.githubusercontent.com/kubernetes/apiextensions-apiserver/master/pkg/apis/apiextensions/v1/generated.proto", "k8s.io/apiextensions-apiserver/pkg/apis/apiextensions/v1/generated.proto"),
    ];

    // Externe Protos herunterladen
    for &(url, relative_path) in &base_urls {
        let dest_path = proto_include.join(relative_path);
        if !dest_path.exists() {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Fehler beim Erstellen des Parent-Ordners für {:?}: {}", dest_path, e))?;
            }
            
            println!("cargo:warning=Lade Abhängigkeit herunter: {}", url);
            let response = ureq::get(url).call()
                .map_err(|e| format!("HTTP-Request fehlgeschlagen für {}: {}", url, e))?;
            
            let content = response.into_body().read_to_string()
                .map_err(|e| format!("Fehler beim Lesen des Bodys von {}: {}", url, e))?;
            
            let mut file = fs::File::create(&dest_path)
                .map_err(|e| format!("Konnte Datei nicht erstellen {:?}: {}", dest_path, e))?;
            file.write_all(content.as_bytes())
                .map_err(|e| format!("Fehler beim Schreiben in Datei {:?}: {}", dest_path, e))?;
        }
    }

    // 2. Struktur für das lokale ArgoCD-Repo innerhalb von proto_include aufbauen
    let argo_project_dir = proto_include.join("github.com/argoproj/argo-cd/v3");
    fs::create_dir_all(&argo_project_dir)
        .map_err(|e| format!("Fehler beim Erstellen von {:?}: {}", argo_project_dir, e))?;

    let local_argo_source = Path::new("argo-cd");
    
    // Validierung, ob das argo-cd Submodul/Verzeichnis überhaupt da ist, wo wir es erwarten
    if !local_argo_source.exists() {
        return Err(format!(
            "Das lokale ArgoCD-Verzeichnis {:?} wurde nicht gefunden! Befindest du dich im Workspace-Root?", 
            local_argo_source
        ).into());
    }

    let proto_folders = ["server", "reposerver", "pkg/apis"];
    for folder in &proto_folders {
        let src = local_argo_source.join(folder);
        let dest = argo_project_dir.join(folder);
        if src.exists() {
            copy_dir_all(&src, &dest)
                .map_err(|e| format!("Fehler beim Kopieren von {:?} nach {:?}: {}", src, dest, e))?;
        } else {
            println!("cargo:warning=Hinweis: Optionaler Ordner {:?} existiert lokal nicht.", src);
        }
    }

    // Cargo Rerun-Triggers
    println!("cargo:rerun-if-changed=argo-cd/server/application/application.proto");
    println!("cargo:rerun-if-changed=argo-cd/server/repository/repository.proto");
    println!("cargo:rerun-if-changed=argo-cd/reposerver/repository/repository.proto");
    println!("cargo:rerun-if-changed=argo-cd/server/events/events.proto");

    tonic_build::configure()
    .compile_well_known_types(true)
    .compile_protos(
        &[
            argo_project_dir.join("server/application/application.proto"),
            argo_project_dir.join("server/repository/repository.proto"),
            argo_project_dir.join("reposerver/repository/repository.proto"),
            argo_project_dir.join("server/events/events.proto"),
        ],
        &[proto_include],
    )
    .map_err(|e| format!("Tonic-Kompilierung fehlgeschlagen: {}", e))?;

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            let entry_path = entry.path();
            // Nur Dateien mit der Endung .proto kopieren
            if entry_path.extension().map_or(false, |ext| ext == "proto") {
                fs::copy(&entry_path, dst.join(entry.file_name()))
                    .map_err(|e| std::io::Error::new(
                        e.kind(),
                        format!("Fehler beim Kopieren von {:?}: {}", entry_path, e)
                    ))?;
            }
        }
    }
    Ok(())
}