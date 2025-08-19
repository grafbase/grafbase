const { createYoga } = require('graphql-yoga');
const { buildSubgraphSchema } = require('@graphql-tools/federation');
const { createServer } = require('http');
const { readFileSync } = require('fs');
const { parse } = require('graphql');

const typeDefs = parse(readFileSync('./schema.graphql', { encoding: 'utf-8' }));

// Test medical records data - in production this would be highly secured
const medicalRecords = [
  {
    id: 'mr-1',
    patientId: 'patient-1',
    createdAt: '2020-01-15T08:00:00Z',
    updatedAt: '2024-01-15T09:30:00Z',
    bloodType: 'O+',
    allergies: ['Penicillin', 'Peanuts'],
    currentMedications: [
      {
        name: 'Lisinopril',
        dosage: '10mg',
        frequency: 'Once daily',
        startedAt: '2023-06-01T00:00:00Z',
        prescribedBy: 'doc-a2'
      }
    ],
    diagnoses: [
      {
        id: 'diag-1',
        code: 'I10',
        description: 'Essential (primary) hypertension',
        diagnosedAt: '2023-06-01T00:00:00Z',
        diagnosedBy: 'doc-a2',
        severity: 'MODERATE',
        status: 'IN_TREATMENT',
        notes: 'Managed with medication and lifestyle changes'
      }
    ],
    prescriptions: [
      {
        id: 'rx-1',
        medicationName: 'Lisinopril',
        dosage: '10mg',
        frequency: 'Once daily',
        duration: '90 days',
        refills: 3,
        prescribedAt: '2023-06-01T00:00:00Z',
        prescribedBy: 'doc-a2',
        instructions: 'Take with water in the morning',
        controlledSubstance: false
      }
    ],
    labResults: [
      {
        id: 'lab-1',
        testName: 'Complete Blood Count',
        testCode: 'CBC',
        value: 'Within normal limits',
        unit: '',
        normalRange: 'See detailed report',
        isAbnormal: false,
        performedAt: '2024-01-15T08:00:00Z',
        orderedBy: 'doc-a1',
        notes: null
      }
    ],
    vitalSigns: [
      {
        recordedAt: '2024-01-15T09:00:00Z',
        bloodPressureSystolic: 128,
        bloodPressureDiastolic: 82,
        heartRate: 72,
        temperature: 98.6,
        respiratoryRate: 16,
        oxygenSaturation: 98,
        weight: 180.5,
        height: 70,
        bmi: 25.9
      }
    ],
    medicalHistory: [
      {
        date: '2019-03-15T00:00:00Z',
        type: 'SURGERY',
        description: 'Appendectomy',
        provider: 'Dr. Johnson',
        facility: 'St. Mary Hospital',
        attachments: []
      }
    ]
  },
  {
    id: 'mr-2',
    patientId: 'patient-2',
    createdAt: '2021-03-22T08:00:00Z',
    updatedAt: '2024-01-15T10:45:00Z',
    bloodType: 'A+',
    allergies: [],
    currentMedications: [],
    diagnoses: [
      {
        id: 'diag-2',
        code: 'E11.9',
        description: 'Type 2 diabetes mellitus without complications',
        diagnosedAt: '2021-03-22T00:00:00Z',
        diagnosedBy: 'doc-b1',
        severity: 'MODERATE',
        status: 'IN_TREATMENT',
        notes: 'Diet controlled'
      }
    ],
    prescriptions: [],
    labResults: [],
    vitalSigns: [
      {
        recordedAt: '2024-01-15T10:00:00Z',
        bloodPressureSystolic: 118,
        bloodPressureDiastolic: 76,
        heartRate: 68,
        temperature: 98.2,
        respiratoryRate: 14,
        oxygenSaturation: 99,
        weight: 145.0,
        height: 65,
        bmi: 24.1
      }
    ],
    medicalHistory: []
  }
];

const accessLogs = [
  {
    timestamp: '2024-01-15T08:30:00Z',
    userId: 'doc-a1',
    action: 'VIEW_MEDICAL_RECORD',
    resourceAccessed: 'patient-1/medical-record',
    ipAddress: '192.168.1.100'
  }
];

const resolvers = {
  Query: {
    accessLog: (_, { patientId }) => accessLogs.filter(log => log.resourceAccessed.includes(patientId))
  },
  Patient: {
    __resolveReference: (ref) => ({ id: ref.id }),
    medicalRecord: (patient) => medicalRecords.find(r => r.patientId === patient.id)
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

server.listen(4004, () => {
  console.log('🏥 Medical Records subgraph ready at http://localhost:4004/');
});