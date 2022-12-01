import { SignJWT } from 'jose'
import {
  createContext,
  PropsWithChildren,
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
  const [token, setToken] = useState('')

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
