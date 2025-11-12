# Test file for CodeQL SARIF test
def get_user_data(user_id):
    query = "SELECT * FROM users WHERE id = " + user_id
    # This would be a SQL injection vulnerability
    return query
