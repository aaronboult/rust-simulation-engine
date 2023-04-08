"""Development server for testing WASM.
"""

import flask

app = flask.Flask(__name__)

@app.route('/')
def index():
    try:
        with open('index.html') as f:
            return f.read()

    except FileNotFoundError:
        return '<h1>index.html not found</h1>'

# Handle static files
@app.route('/<path:path>')
def static_file(path):
    return flask.send_from_directory('.', path)

# Fix mimetype for .js and .wasm
@app.after_request
def after_request(response):
    # Check if Content-Disposition header is set
    if 'Content-Disposition' not in response.headers:
        return response

    # Check if we are serving a .js file based on name
    if response.headers['Content-Disposition'].endswith('.js'):
        response.headers['Content-Type'] = 'text/javascript'

    # Check if we are serving a .wasm file based on name
    if response.headers['Content-Disposition'].endswith('.wasm'):
        response.headers['Content-Type'] = 'application/wasm'

    return response

if __name__ == '__main__':
    app.run(debug=True)