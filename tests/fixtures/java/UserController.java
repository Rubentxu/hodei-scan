package com.example.controller;

import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api/users")
public class UserController {

    @PostMapping
    // TODO: Add validation
    public void createUser(@RequestBody String user) {
        // TODO: Implement user creation
        System.out.println("Creating user: " + user);
    }

    @GetMapping("/{id}")
    public String getUser(@PathVariable Long id) {
        // FIXME: Add error handling
        return "User " + id;
    }

    @GetMapping
    public String getAllUsers() {
        // HACK: Temporary implementation
        return "[]";
    }
}
