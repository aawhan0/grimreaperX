# Useful Commands

## Desktop

```powershell
cd desktop
npm install
npm run tauri dev
```

Rust tests:

```powershell
cd desktop/src-tauri
cargo test
```

## Backend

```powershell
cd backend
python -m venv .venv
.venv\Scripts\activate
pip install -r requirements.txt
uvicorn app.main:app --reload
```

## Android

Open `android/` in Android Studio and let Gradle sync. Build with the Android Studio UI first; then move to `gradlew assembleDebug` once the environment is confirmed.
