# Architecture Rules

This document defines mandatory architecture patterns and conventions for this project.

## 1. Project Structure

### Directory Organization

```
project/
├── src/                    # Source code
│   ├── main/              # Electron main process (if applicable)
│   ├── renderer/          # Frontend code
│   │   ├── components/    # Reusable UI components
│   │   ├── pages/         # Page components
│   │   ├── hooks/         # Custom hooks
│   │   ├── services/      # API services
│   │   ├── stores/        # State management
│   │   ├── types/         # TypeScript types
│   │   └── utils/         # Utility functions
│   └── shared/            # Code shared between main/renderer
├── docs/                   # Documentation
│   ├── ARCHITECTURE.md    # Architecture documentation
│   └── API.md             # API documentation
├── planning/              # Planning files (Manus-style)
│   ├── task_plan.md       # Current task plan
│   ├── progress.md        # Progress tracking
│   └── findings.md        # Research findings
├── tests/                 # Test files
└── .claude/               # Claude configuration
    ├── commands/          # Custom commands
    ├── skills/            # Skill definitions
    ├── hooks/             # Hook scripts
    └── rules/             # Project rules
```

## 2. Code Conventions

### TypeScript

- **Strict Mode**: All projects MUST use TypeScript strict mode
- **No Type Suppression**: NEVER use `as any`, `@ts-ignore`, `@ts-expect-error`
- **Explicit Types**: Export types for all public interfaces
- **Type-First Design**: Define types before implementation

```typescript
// ✅ GOOD: Explicit interface
interface UserConfig {
  name: string;
  email: string;
  preferences: UserPreferences;
}

// ❌ BAD: Type suppression
const user = data as any;
```

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Files (components) | PascalCase | `UserProfile.tsx` |
| Files (utilities) | camelCase | `formatDate.ts` |
| Files (types) | camelCase + .types | `user.types.ts` |
| Components | PascalCase | `function UserProfile()` |
| Functions | camelCase | `function getUserById()` |
| Constants | SCREAMING_SNAKE | `const MAX_RETRIES = 3` |
| Types/Interfaces | PascalCase | `interface UserConfig` |

### Import Order

```typescript
// 1. External libraries
import React from 'react';
import { useQuery } from '@tanstack/react-query';

// 2. Internal absolute imports
import { UserService } from '@/services/UserService';
import { Button } from '@/components/ui/Button';

// 3. Relative imports
import { formatName } from './utils';
import type { UserProps } from './types';
```

## 3. Component Architecture

### Component Structure

```typescript
// 1. Types at the top
interface ComponentProps {
  title: string;
  onAction: () => void;
}

// 2. Component definition
export function Component({ title, onAction }: ComponentProps) {
  // 3. Hooks first
  const [state, setState] = useState<string>('');
  const queryResult = useQuery(/* ... */);
  
  // 4. Derived state
  const isValid = state.length > 0;
  
  // 5. Event handlers
  const handleClick = useCallback(() => {
    onAction();
  }, [onAction]);
  
  // 6. Effects
  useEffect(() => {
    // Side effects
  }, [dependency]);
  
  // 7. Early returns
  if (!queryResult.data) {
    return <LoadingState />;
  }
  
  // 8. Main render
  return (
    <div>
      {/* JSX */}
    </div>
  );
}
```

### Colocation Rules

- Keep related code together (component, styles, types, tests)
- Shared code lives in parent directory
- Domain-specific code stays in domain folder

## 4. State Management

### Local vs Global State

| Use Local State | Use Global State |
|-----------------|------------------|
| Form input values | User session |
| UI toggle states | App configuration |
| Component-specific data | Cross-component data |
| Temporary values | Cached API data |

### State Update Patterns

```typescript
// ✅ GOOD: Immutable update
setState(prev => [...prev, newItem]);

// ❌ BAD: Direct mutation
state.push(newItem);
setState(state);
```

## 5. Error Handling

### Required Error Boundaries

- Wrap major page sections
- Provide meaningful fallback UI
- Log errors for debugging

### API Error Handling

```typescript
// ✅ GOOD: Proper error handling
try {
  const result = await api.fetchData();
  return result;
} catch (error) {
  if (error instanceof NetworkError) {
    // Handle network error
  } else if (error instanceof ValidationError) {
    // Handle validation error
  }
  throw error; // Re-throw unknown errors
}

// ❌ BAD: Silent failure
try {
  return await api.fetchData();
} catch (error) {
  return null;
}
```

## 6. Documentation Requirements

### Required Documentation

1. **README.md** - Project overview and setup
2. **docs/ARCHITECTURE.md** - System architecture
3. **planning/task_plan.md** - Current task planning
4. **Code Comments** - Complex logic explanation

### JSDoc for Public APIs

```typescript
/**
 * Fetches user data by ID
 * @param userId - The unique user identifier
 * @returns User object or null if not found
 * @throws {NetworkError} When API is unreachable
 */
export async function getUserById(userId: string): Promise<User | null> {
  // ...
}
```

## 7. Testing Standards

### Test File Organization

```
src/
├── components/
│   ├── Button/
│   │   ├── Button.tsx
│   │   ├── Button.test.tsx    # Unit tests
│   │   └── Button.stories.tsx  # Storybook (if applicable)
```

### Test Coverage Requirements

- Critical business logic: 80%+
- Utility functions: 90%+
- UI components: Behavioral testing

## 8. Performance Guidelines

### Optimization Rules

1. **Memoization**: Use `useMemo`/`useCallback` for expensive computations
2. **Code Splitting**: Lazy load routes and heavy components
3. **Virtual Lists**: Use virtualization for long lists (100+ items)
4. **Image Optimization**: Use next/image or similar

### What NOT to Optimize Prematurely

- Simple value comparisons
- Small arrays (<100 items)
- Infrequently rendered components

## 9. Security Rules

### Never Commit

- API keys or secrets
- Environment files with real values
- Credentials or passwords
- Personal data

### Input Validation

- Validate all user inputs
- Sanitize data before storage
- Use parameterized queries

## 10. Git Conventions

### Commit Message Format

```
type(scope): description

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

### Example

```
feat(auth): add JWT token refresh mechanism

- Implement automatic token refresh 5 minutes before expiry
- Add refresh token rotation for security
- Store tokens in httpOnly cookies

Closes #123
```
