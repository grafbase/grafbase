const { createYoga } = require('graphql-yoga');
const { buildSubgraphSchema } = require('@graphql-tools/federation');
const { createServer } = require('http');
const { readFileSync } = require('fs');
const { parse } = require('graphql');

const typeDefs = parse(readFileSync('./schema.graphql', { encoding: 'utf-8' }));

// Test data - in production this would come from a database
const patients = [
  {
    id: 'patient-1',
    firstName: 'John',
    lastName: 'Doe',
    dateOfBirth: '1985-03-15',
    email: 'john.doe@email.com',
    phone: '555-0101',
    ssn: '123-45-6789',
    insuranceProvider: 'HealthCare Plus',
    insurancePolicyNumber: 'HCP-789456',
    emergencyContactName: 'Jane Doe',
    emergencyContactPhone: '555-0102',
    emergencyContactRelation: 'Spouse'
  },
  {
    id: 'patient-2',
    firstName: 'Sarah',
    lastName: 'Johnson',
    dateOfBirth: '1990-07-22',
    email: 'sarah.j@email.com',
    phone: '555-0201',
    ssn: '987-65-4321',
    insuranceProvider: 'MediShield',
    insurancePolicyNumber: 'MS-123789',
    emergencyContactName: 'Robert Johnson',
    emergencyContactPhone: '555-0202',
    emergencyContactRelation: 'Father'
  },
  {
    id: 'patient-3',
    firstName: 'Michael',
    lastName: 'Chen',
    dateOfBirth: '1978-11-30',
    email: 'mchen@email.com',
    phone: '555-0301',
    ssn: '456-78-9123',
    insuranceProvider: 'Global Health',
    insurancePolicyNumber: 'GH-456123',
    emergencyContactName: 'Lisa Chen',
    emergencyContactPhone: '555-0302',
    emergencyContactRelation: 'Sister'
  },
  {
    id: 'patient-4',
    firstName: 'Emily',
    lastName: 'Rodriguez',
    dateOfBirth: '1995-05-18',
    email: 'emily.r@email.com',
    phone: '555-0401',
    ssn: '321-54-9876',
    insuranceProvider: 'HealthCare Plus',
    insurancePolicyNumber: 'HCP-852963',
    emergencyContactName: 'Carlos Rodriguez',
    emergencyContactPhone: '555-0402',
    emergencyContactRelation: 'Brother'
  }
];

const resolvers = {
  Query: {
    patient: (_, { id }) => patients.find(p => p.id === id),
    patients: () => patients,
    patientBySsn: (_, { ssn }) => patients.find(p => p.ssn === ssn)
  },
  Patient: {
    __resolveReference: (ref) => patients.find(p => p.id === ref.id)
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

server.listen(4001, () => {
  console.log('🏥 Patients subgraph ready at http://localhost:4001/');
});