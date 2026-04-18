# Contoh File Konfigurasi — App Blocker

Direktori ini berisi contoh file JSON/TOML/YAML untuk mengelola
blacklist dan whitelist melalui CLI App Blocker.

---

## Daftar File

| File | Format | Kegunaan |
|---|---|---|
| `blacklist_single.json`   | JSON | Tambah 1 aplikasi ke blacklist |
| `blacklist_bulk.json`     | JSON | Tambah banyak aplikasi sekaligus |
| `blacklist_template.toml` | TOML | Template kosong blacklist |
| `whitelist_single.json`   | JSON | Tambah 1 aplikasi ke whitelist |
| `whitelist_bulk.json`     | JSON | Whitelist tool pendidikan lab lengkap |
| `whitelist_template.yaml` | YAML | Template kosong whitelist |

---

## Cara Penggunaan

### Tambah Blacklist

```powershell
# Dari file JSON (1 aplikasi)
app_blocker.exe add-blacklist --file examples\blacklist_single.json

# Dari file JSON (banyak sekaligus)
app_blocker.exe add-blacklist --file examples\blacklist_bulk.json

# Dari file TOML
app_blocker.exe add-blacklist --file examples\blacklist_template.toml

# Langsung dari CLI (tanpa file)
app_blocker.exe add-blacklist --name "game.exe" --app-name "Nama Game"
```

### Tambah Whitelist

```powershell
# Whitelist tool lab lengkap (20+ aplikasi pendidikan)
app_blocker.exe add-whitelist --file examples\whitelist_bulk.json

# Whitelist 1 aplikasi
app_blocker.exe add-whitelist --file examples\whitelist_single.json

# Dari YAML
app_blocker.exe add-whitelist --file examples\whitelist_template.yaml

# Langsung dari CLI
app_blocker.exe add-whitelist --name "Code.exe"
```

### Lihat Daftar

```powershell
app_blocker.exe list-blacklist
app_blocker.exe list-whitelist
```

### Hapus dari Daftar

```powershell
app_blocker.exe remove-blacklist steam.exe
app_blocker.exe remove-whitelist Code.exe
```

---

## Format JSON — Blacklist

### Satu Aplikasi
```json
{
  "name": "Nama Game",
  "process_names": ["game.exe", "launcher.exe"],
  "paths": ["C:\\Games\\NamaGame\\"],
  "description": "Keterangan"
}
```

### Banyak Aplikasi (Bulk)
```json
{
  "entries": [
    {
      "name": "Game A",
      "process_names": ["gameA.exe"],
      "paths": [],
      "description": "Game A"
    },
    {
      "name": "Game B",
      "process_names": ["gameB.exe"],
      "paths": ["C:\\Games\\B\\"],
      "description": "Game B"
    }
  ]
}
```

---

## Format JSON — Whitelist

### Satu Aplikasi
```json
{
  "name": "Visual Studio Code",
  "process_names": ["Code.exe"],
  "paths": ["C:\\Program Files\\Microsoft VS Code\\"],
  "description": "Editor kode"
}
```

### Banyak Aplikasi (Bulk)
```json
{
  "entries": [
    {
      "name": "VS Code",
      "process_names": ["Code.exe"],
      "paths": [],
      "description": "Editor"
    }
  ]
}
```

---

## Tips

- **`process_names`** dicocokkan secara **case-insensitive**
- **`paths`** mendukung wildcard `*` (misal: `C:\Users\*\AppData\...`)
- **Whitelist selalu prioritas lebih tinggi** dari blacklist
- Setelah menambah/menghapus, App Blocker **tidak perlu di-restart**
  (hot reload aktif otomatis)
- Field `description` bersifat opsional tapi sangat disarankan
  untuk memudahkan audit log

---

*App Blocker — Dikembangkan oleh Muhamad Fahmi, Asisten Kepala Lab Komputer*
