import { SignJWT } from 'jose'
import {
  createContext,
  PropsWithChildren,
  useCallback,
  useContext,
  useEffect,
  useState
} from 'react'

const issuerUrl = import.meta.env.VITE_ISSUER_URL
const secret = new Uint8Array(
  (import.meta.env.VITE_JWT_SECRET as string)
    .split('')
    .map((c) => c.charCodeAt(0))
)

const getToken = (role: string) => {
  const groups = role ? [role] : []
  return new SignJWT({ sub: 'user_1234', groups })
    .setProtectedHeader({ alg: 'HS256', typ: 'JWT' })
    .setIssuer(issuerUrl)
    .setIssuedAt()
    .setExpirationTime('2h')
    .sign(secret)
}

const AuthContext = createContext({
  token: '',
  setRole: (role: string) => {}
})

export const useAuth = () => useContext(AuthContext)

export const AuthProvider = ({ children }: PropsWithChildren) => {
  const [role, setRole] = useState('')
  const [token, _setToken] = useState('')

  const handleStorageChange = useCallback((event: StorageEvent) => {
    if (event.storageArea === localStorage && event.key === 'token') {
      _setToken(event.newValue ?? '')
    }
  }, [])

  const setToken = useCallback(
    (token: string) => {
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('token', token)
        _setToken(token)
      }
    },
    [typeof localStorage]
  )

  useEffect(() => {
    if (typeof window !== 'undefined') {
      window.addEventListener('storage', handleStorageChange)
      _setToken(localStorage.getItem('token') ?? '')
    }

    return () => {
      if (typeof window !== 'undefined') {
        window.removeEventListener('storage', handleStorageChange)
      }
    }
  }, [])

  useEffect(() => {
    const init = async () => {
      const token = await getToken(role)
      setToken(token)
    }
    init()
  }, [role])

  return (
    <AuthContext.Provider value={{ token, setRole }}>
      {children}
    </AuthContext.Provider>
  )
}
