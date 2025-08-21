const { createYoga } = require('graphql-yoga');
const { buildSubgraphSchema } = require('@graphql-tools/federation');
const { createServer } = require('http');
const { readFileSync } = require('fs');
const { parse } = require('graphql');

const typeDefs = parse(readFileSync('./schema.graphql', { encoding: 'utf-8' }));

// Test data for Clinic B - Westside Health Clinic
const clinicInfo = {
  id: 'clinic-b',
  name: 'Westside Health Clinic',
  address: '456 Oak Avenue',
  city: 'San Francisco',
  state: 'CA',
  zipCode: '94102',
  phone: '555-1000',
  operatingHours: [
    { dayOfWeek: 'MONDAY', openTime: '08:00', closeTime: '18:00' },
    { dayOfWeek: 'TUESDAY', openTime: '08:00', closeTime: '18:00' },
    { dayOfWeek: 'WEDNESDAY', openTime: '08:00', closeTime: '18:00' },
    { dayOfWeek: 'THURSDAY', openTime: '08:00', closeTime: '18:00' },
    { dayOfWeek: 'FRIDAY', openTime: '08:00', closeTime: '17:00' },
    { dayOfWeek: 'SATURDAY', openTime: '09:00', closeTime: '13:00' }
  ],
  services: [
    'General Practice',
    'Cardiology',
    'Dermatology',
    'Pediatrics',
    'X-Ray',
    'Blood Tests',
    'Vaccinations'
  ]
};

const doctors = [
  {
    id: 'doc-b1',
    firstName: 'Maria',
    lastName: 'Garcia',
    specialty: 'General Practice',
    licenseNumber: 'CA-98765',
    email: 'm.garcia@westside-health.com',
    phone: '555-1001',
    clinicId: 'clinic-b'
  },
  {
    id: 'doc-b2',
    firstName: 'David',
    lastName: 'Thompson',
    specialty: 'Orthopedics',
    licenseNumber: 'CA-98766',
    email: 'd.thompson@westside-health.com',
    phone: '555-1002',
    clinicId: 'clinic-b'
  },
  {
    id: 'doc-b3',
    firstName: 'Jennifer',
    lastName: 'Park',
    specialty: 'Neurology',
    licenseNumber: 'CA-98767',
    email: 'j.park@westside-health.com',
    phone: '555-1003',
    clinicId: 'clinic-b'
  },
  {
    id: 'doc-b4',
    firstName: 'Ahmed',
    lastName: 'Hassan',
    specialty: 'Emergency Medicine',
    licenseNumber: 'CA-98768',
    email: 'a.hassan@westside-health.com',
    phone: '555-1004',
    clinicId: 'clinic-b'
  }
];

const appointments = [
  {
    id: 'apt-b1',
    patientId: 'patient-1',
    doctorId: 'doc-b1',
    scheduledAt: '2024-01-20T09:00:00Z',
    duration: 30,
    status: 'SCHEDULED',
    reason: 'Annual checkup',
    notes: 'Patient requested blood work',
    clinicId: 'clinic-b'
  },
  {
    id: 'apt-b2',
    patientId: 'patient-1',
    doctorId: 'doc-b2',
    scheduledAt: '2024-01-15T14:00:00Z',
    duration: 45,
    status: 'COMPLETED',
    reason: 'Follow-up for hypertension',
    notes: 'Blood pressure improved',
    clinicId: 'clinic-b'
  },
  {
    id: 'apt-b3',
    patientId: 'patient-3',
    doctorId: 'doc-b3',
    scheduledAt: '2024-01-22T10:30:00Z',
    duration: 30,
    status: 'SCHEDULED',
    reason: 'Pediatric consultation for child',
    notes: null,
    clinicId: 'clinic-b'
  }
];

const resolvers = {
  Query: {
    clinic: (_, { id }) => id === clinicInfo.id ? clinicInfo : null,
    clinics: () => [clinicInfo],
    doctors: () => doctors,
    doctor: (_, { id }) => doctors.find(d => d.id === id),
    appointments: () => appointments,
    appointment: (_, { id }) => appointments.find(a => a.id === id),
    appointmentsByPatient: (_, { patientId }) => appointments.filter(a => a.patientId === patientId),
    appointmentsByDoctor: (_, { doctorId }) => appointments.filter(a => a.doctorId === doctorId),
    appointmentsByDate: (_, { date }) => appointments.filter(a => a.scheduledAt.startsWith(date))
  },
  Doctor: {
    __resolveReference: (ref) => doctors.find(d => d.id === ref.id)
  },
  Appointment: {
    __resolveReference: (ref) => appointments.find(a => a.id === ref.id),
    patient: (appointment) => ({ id: appointment.patientId }),
    doctor: (appointment) => doctors.find(d => d.id === appointment.doctorId)
  },
  Clinic: {
    __resolveReference: (ref) => ref.id === clinicInfo.id ? clinicInfo : null
  }
};

const schema = buildSubgraphSchema({
  typeDefs,
  resolvers
});

const yoga = createYoga({
  schema,
  landingPage: false,
  maskedErrors: false,
  graphiql: {
    endpoint: '/'
  }
});

const server = createServer(yoga);

server.listen(4003, () => {
  console.log('🏥 Clinic B subgraph ready at http://localhost:4003/');
});