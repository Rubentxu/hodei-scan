# Épica Web 6: Auth & RBAC
## Autenticación y Control de Acceso

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 3 (Meses 13-24)
**Prioridad:** 🔴 Critical

---

## 📋 Resumen Ejecutivo

Sistema completo de autenticación y autorización con roles y permisos granulares para multi-tenant organizations.

**Objetivos:**
- ✅ JWT authentication
- ✅ Login/Logout
- ✅ Role-based access control (RBAC)
- ✅ Organization management
- ✅ User management
- ✅ Permission guards
- ✅ SSO integration ready
- ✅ Session management

---

## 👥 Historias de Usuario

### US-WEB-12: Como user, quiero login securely

**Prioridad:** 🔴 Critical
**Story Points:** 8

```gherkin
Feature: Authentication
  Como user
  Quiero login to hodei-scan
  Para access my projects

  Scenario: Successful login
    Given valid credentials
    When makes login
    Then debería:
      And redirect to dashboard
      And store JWT token
      And show user name en header
      And show logout option

  Scenario: Invalid credentials
    Given invalid credentials
    When makes login
    Then debería:
      And show error message
      And NOT redirect
      And clear form

  Scenario: Protected route
    Given user NOT authenticated
    When navigates to protected page
    Then debería redirect to login
```

**Tareas:**

1. **TASK-WEB-06-01: Auth Context** (2 días)
2. **TASK-WEB-06-02: Login Page** (2 días)
3. **TASK-WEB-06-03: Protected Routes** (2 días)
4. **TASK-WEB-06-04: Role Guards** (1 día)
5. **TASK-WEB-06-05: User Management UI** (3 días)

**Tests:**

```typescript
describe('Authentication', () => {
  it('should login with valid credentials', async () => {
    render(<LoginPage />);
    
    fireEvent.change(screen.getByLabelText('Email'), {
      target: { value: 'user@example.com' },
    });
    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'password' },
    });
    
    fireEvent.click(screen.getByText('Login'));
    
    await waitFor(() => {
      expect(screen.getByText('Dashboard')).toBeInTheDocument();
    });
  });

  it('should redirect on protected route without auth', () => {
    render(
      <BrowserRouter>
        <ProtectedRoute>
          <Dashboard />
        </ProtectedRoute>
      </BrowserRouter>
    );
    
    expect(screen.getByText('Login')).toBeInTheDocument();
  });

  it('should restrict access based on role', () => {
    const user = { role: 'viewer' };
    
    render(
      <RoleGuard requiredRole="admin">
        <AdminPanel />
      </RoleGuard>
    );
    
    expect(screen.getByText('Access Denied')).toBeInTheDocument();
  });
});
```

---

## 🏗️ Auth Architecture

```typescript
// contexts/AuthContext.tsx
interface AuthContextType {
  user: User | null;
  token: string | null;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
  hasPermission: (permission: string) => boolean;
}

export const AuthContext = createContext<AuthContextType | undefined>(undefined);

// hooks/useAuth.ts
export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
};

// components/ProtectedRoute.tsx
export const ProtectedRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { user } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    if (!user) {
      navigate('/login');
    }
  }, [user, navigate]);

  if (!user) {
    return <LoadingSpinner />;
  }

  return <>{children}</>;
};
```

---

## 🔄 Criterios de Done

- [ ] ✅ Login/Logout working
- [ ] ✅ JWT token management
- [ ] ✅ Protected routes
- [ ] ✅ Role-based access control
- [ ] ✅ User management interface
- [ ] ✅ Permission guards
- [ ] ✅ Session persistence
- [ ] ✅ 100% tests

**Total Story Points:** 52 | **Duración:** 6 semanas
