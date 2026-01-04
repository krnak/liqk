pub const LOGIN_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Oxigraph Gate - Login</title>
    <style>
        * {
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a2e;
            color: #eee;
            min-height: 100vh;
            margin: 0;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .container {
            text-align: center;
            background: #16213e;
            padding: 3rem;
            border-radius: 12px;
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
            max-width: 400px;
            width: 90%;
        }
        h1 {
            margin: 0 0 0.5rem 0;
            color: #e94560;
            font-size: 1.8rem;
        }
        p {
            margin: 0 0 2rem 0;
            color: #aaa;
        }
        input[type="text"] {
            width: 100%;
            padding: 0.875rem;
            font-size: 1rem;
            font-family: monospace;
            border: 2px solid #0f3460;
            border-radius: 6px;
            background: #1a1a2e;
            color: #eee;
            text-align: center;
            margin-bottom: 1rem;
        }
        input[type="text"]:focus {
            outline: none;
            border-color: #e94560;
        }
        button {
            width: 100%;
            padding: 0.875rem;
            font-size: 1rem;
            font-weight: 600;
            background: #e94560;
            color: white;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            transition: background 0.2s;
        }
        button:hover {
            background: #ff6b6b;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Oxigraph Gate</h1>
        <p>Enter your access token to continue</p>
        <form method="POST" action="/gate/login">
            <input type="text" name="token" placeholder="Access Token" autocomplete="off" required>
            <button type="submit">Authenticate</button>
        </form>
    </div>
</body>
</html>
"#;

pub const LOGIN_ERROR_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Oxigraph Gate - Login Failed</title>
    <style>
        * {
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a2e;
            color: #eee;
            min-height: 100vh;
            margin: 0;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .container {
            text-align: center;
            background: #16213e;
            padding: 3rem;
            border-radius: 12px;
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
            max-width: 400px;
            width: 90%;
        }
        h1 {
            margin: 0 0 0.5rem 0;
            color: #e94560;
            font-size: 1.8rem;
        }
        p {
            margin: 0 0 2rem 0;
            color: #ff6b6b;
        }
        input[type="text"] {
            width: 100%;
            padding: 0.875rem;
            font-size: 1rem;
            font-family: monospace;
            border: 2px solid #e94560;
            border-radius: 6px;
            background: #1a1a2e;
            color: #eee;
            text-align: center;
            margin-bottom: 1rem;
        }
        input[type="text"]:focus {
            outline: none;
            border-color: #e94560;
        }
        button {
            width: 100%;
            padding: 0.875rem;
            font-size: 1rem;
            font-weight: 600;
            background: #e94560;
            color: white;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            transition: background 0.2s;
        }
        button:hover {
            background: #ff6b6b;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Oxigraph Gate</h1>
        <p>Invalid token. Please try again.</p>
        <form method="POST" action="/gate/login">
            <input type="text" name="token" placeholder="Access Token" autocomplete="off" required>
            <button type="submit">Authenticate</button>
        </form>
    </div>
</body>
</html>
"#;

pub const UPLOAD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Oxigraph Gate - Upload</title>
    <style>
        * {
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a2e;
            color: #eee;
            min-height: 100vh;
            margin: 0;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .container {
            text-align: center;
            background: #16213e;
            padding: 3rem;
            border-radius: 12px;
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
            max-width: 500px;
            width: 90%;
        }
        h1 {
            margin: 0 0 0.5rem 0;
            color: #e94560;
            font-size: 1.8rem;
        }
        p {
            margin: 0 0 2rem 0;
            color: #aaa;
        }
        .drop-zone {
            border: 2px dashed #0f3460;
            border-radius: 8px;
            padding: 2rem;
            margin-bottom: 1rem;
            transition: border-color 0.2s, background 0.2s;
            cursor: pointer;
        }
        .drop-zone:hover, .drop-zone.dragover {
            border-color: #e94560;
            background: rgba(233, 69, 96, 0.1);
        }
        .drop-zone p {
            margin: 0;
            color: #888;
        }
        input[type="file"] {
            display: none;
        }
        .file-list {
            text-align: left;
            margin-bottom: 1rem;
            max-height: 150px;
            overflow-y: auto;
        }
        .file-item {
            padding: 0.5rem;
            background: #1a1a2e;
            border-radius: 4px;
            margin-bottom: 0.5rem;
            font-size: 0.9rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .file-item .size {
            color: #888;
            font-size: 0.8rem;
        }
        button {
            width: 100%;
            padding: 0.875rem;
            font-size: 1rem;
            font-weight: 600;
            background: #e94560;
            color: white;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            transition: background 0.2s;
        }
        button:hover {
            background: #ff6b6b;
        }
        button:disabled {
            background: #555;
            cursor: not-allowed;
        }
        .progress {
            display: none;
            margin-top: 1rem;
        }
        .progress-bar {
            height: 8px;
            background: #0f3460;
            border-radius: 4px;
            overflow: hidden;
        }
        .progress-fill {
            height: 100%;
            background: #4ade80;
            width: 0%;
            transition: width 0.3s;
        }
        .progress-text {
            margin-top: 0.5rem;
            font-size: 0.9rem;
            color: #888;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>File Upload</h1>
        <p>Upload files to the server (max 4 GB)</p>
        <form id="uploadForm" method="POST" action="/upload" enctype="multipart/form-data">
            <div class="drop-zone" id="dropZone">
                <p>Drop files here or click to select</p>
                <input type="file" name="files" id="fileInput" multiple>
            </div>
            <div class="file-list" id="fileList"></div>
            <button type="submit" id="uploadBtn" disabled>Upload Files</button>
            <div class="progress" id="progress">
                <div class="progress-bar">
                    <div class="progress-fill" id="progressFill"></div>
                </div>
                <div class="progress-text" id="progressText">Uploading...</div>
            </div>
        </form>
    </div>
    <script>
        const dropZone = document.getElementById('dropZone');
        const fileInput = document.getElementById('fileInput');
        const fileList = document.getElementById('fileList');
        const uploadBtn = document.getElementById('uploadBtn');
        const form = document.getElementById('uploadForm');
        const progress = document.getElementById('progress');
        const progressFill = document.getElementById('progressFill');
        const progressText = document.getElementById('progressText');

        function formatSize(bytes) {
            if (bytes < 1024) return bytes + ' B';
            if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
            if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
            return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB';
        }

        function updateFileList() {
            fileList.innerHTML = '';
            for (const file of fileInput.files) {
                const div = document.createElement('div');
                div.className = 'file-item';
                div.innerHTML = `<span>${file.name}</span><span class="size">${formatSize(file.size)}</span>`;
                fileList.appendChild(div);
            }
            uploadBtn.disabled = fileInput.files.length === 0;
        }

        dropZone.addEventListener('click', () => fileInput.click());
        dropZone.addEventListener('dragover', (e) => {
            e.preventDefault();
            dropZone.classList.add('dragover');
        });
        dropZone.addEventListener('dragleave', () => dropZone.classList.remove('dragover'));
        dropZone.addEventListener('drop', (e) => {
            e.preventDefault();
            dropZone.classList.remove('dragover');
            fileInput.files = e.dataTransfer.files;
            updateFileList();
        });
        fileInput.addEventListener('change', updateFileList);

        form.addEventListener('submit', (e) => {
            e.preventDefault();
            const formData = new FormData(form);
            const xhr = new XMLHttpRequest();

            progress.style.display = 'block';
            uploadBtn.disabled = true;

            xhr.upload.addEventListener('progress', (e) => {
                if (e.lengthComputable) {
                    const percent = (e.loaded / e.total) * 100;
                    progressFill.style.width = percent + '%';
                    progressText.textContent = `Uploading... ${formatSize(e.loaded)} / ${formatSize(e.total)}`;
                }
            });

            xhr.addEventListener('load', () => {
                if (xhr.status === 200) {
                    document.body.innerHTML = xhr.responseText;
                } else {
                    progressText.textContent = 'Upload failed: ' + xhr.statusText;
                    uploadBtn.disabled = false;
                }
            });

            xhr.addEventListener('error', () => {
                progressText.textContent = 'Upload failed';
                uploadBtn.disabled = false;
            });

            xhr.open('POST', '/upload');
            xhr.send(formData);
        });
    </script>
</body>
</html>
"#;

pub fn upload_success_html(message: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Upload Complete</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a2e;
            color: #eee;
            min-height: 100vh;
            margin: 0;
            display: flex;
            align-items: center;
            justify-content: center;
        }}
        .container {{
            text-align: center;
            background: #16213e;
            padding: 3rem;
            border-radius: 12px;
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
            max-width: 500px;
            width: 90%;
        }}
        h1 {{
            margin: 0 0 1rem 0;
            color: #4ade80;
            font-size: 1.8rem;
        }}
        p {{
            margin: 0 0 2rem 0;
            color: #aaa;
        }}
        a {{
            display: inline-block;
            padding: 0.875rem 2rem;
            font-size: 1rem;
            font-weight: 600;
            background: #e94560;
            color: white;
            text-decoration: none;
            border-radius: 6px;
            transition: background 0.2s;
        }}
        a:hover {{
            background: #ff6b6b;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Upload Complete</h1>
        <p>{}</p>
        <a href="/upload">Upload More</a>
    </div>
</body>
</html>"#,
        message
    )
}
