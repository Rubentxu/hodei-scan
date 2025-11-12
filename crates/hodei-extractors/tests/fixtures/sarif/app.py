# Test file for Semgrep SARIF test
def process_template(user_input):
    # Potential template injection vulnerability
    return render_template_string("<h1>" + user_input + "</h1>")
