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

