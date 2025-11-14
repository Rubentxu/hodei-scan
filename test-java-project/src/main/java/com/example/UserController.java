package com.example;
import java.sql.*;
public class UserController {
    private static final String DB_PASSWORD = "hardcoded123";
    public void login(String username, String password) throws SQLException {
        Connection conn = DriverManager.getConnection("jdbc:mysql://localhost", "admin", DB_PASSWORD);
        String query = "SELECT * FROM users WHERE username = '" + username + "'";
        Statement stmt = conn.createStatement();
        stmt.executeQuery(query);
    }
}
