const { createYoga } = require('graphql-yoga');
const { buildSubgraphSchema } = require('@graphql-tools/federation');
const { createServer } = require('http');
const { readFileSync } = require('fs');
const { parse } = require('graphql');

const typeDefs = parse(readFileSync('./schema.graphql', { encoding: 'utf-8' }));

// Test data for Clinic A - Downtown Medical Center
const clinicInfo = {
  id: 'clinic-a',
  name: 'Downtown Medical Center',
  address: '123 Main Street',
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
    id: 'doc-a1',
    firstName: 'Robert',
    lastName: 'Smith',
    specialty: 'General Practice',
    licenseNumber: 'CA-12345',
    email: 'r.smith@downtown-med.com',
    phone: '555-1001',
    clinicId: 'clinic-a'
  },
  {
    id: 'doc-a2',
    firstName: 'Lisa',
    lastName: 'Anderson',
    specialty: 'Cardiology',
    licenseNumber: 'CA-12346',
    email: 'l.anderson@downtown-med.com',
    phone: '555-1002',
    clinicId: 'clinic-a'
  },
  {
    id: 'doc-a3',
    firstName: 'James',
    lastName: 'Wilson',
    specialty: 'Pediatrics',
    licenseNumber: 'CA-12347',
    email: 'j.wilson@downtown-med.com',
    phone: '555-1003',
    clinicId: 'clinic-a'
  }
];

const appointments = [
  {
    id: 'apt-a1',
    patientId: 'patient-1',
    doctorId: 'doc-a1',
    scheduledAt: '2024-01-20T09:00:00Z',
    duration: 30,
    status: 'SCHEDULED',
    reason: 'Annual checkup',
    notes: 'Patient requested blood work',
    clinicId: 'clinic-a'
  },
  {
    id: 'apt-a2',
    patientId: 'patient-1',
    doctorId: 'doc-a2',
    scheduledAt: '2024-01-15T14:00:00Z',
    duration: 45,
    status: 'COMPLETED',
    reason: 'Follow-up for hypertension',
    notes: 'Blood pressure improved',
    clinicId: 'clinic-a'
  },
  {
    id: 'apt-a3',
    patientId: 'patient-3',
    doctorId: 'doc-a3',
    scheduledAt: '2024-01-22T10:30:00Z',
    duration: 30,
    status: 'SCHEDULED',
    reason: 'Pediatric consultation for child',
    notes: null,
    clinicId: 'clinic-a'
  }
];

const resolvers = {
  Query: {
    clinic: (_, { id }) => id === clinicInfo.id ? clinicInfo : null,
    clinics: () => [clinicInfo],
    doctors: () => doctors,
    doctor: (_, { id }) => doctors.find(d => d.id === id)
  },
  Patient: {
    __resolveReference: (ref) => ({ id: ref.id }),
    appointments: (patient) => appointments.filter(a => a.patientId === patient.id)
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

server.listen(4002, () => {
  console.log('🏥 Clinic A subgraph ready at http://localhost:4002/');
});