# Dynamic Schema Contracts Integration Tests
# This file combines all integration tests for the dynamic-schema-contracts example

################################################################################
# Basic Healthcare System Tests
################################################################################

# Test 1: Query patient basic information
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: patient-123
```json
{
  "query": "query GetPatient { patient(id: \"patient-1\") { id firstName lastName email phone } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.firstName" == "John"
jsonpath "$.data.patient.lastName" == "Doe"
jsonpath "$.data.patient.email" == "john.doe@email.com"

# Test 2: Query all patients
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: doctor-789
```json
{
  "query": "query GetAllPatients { patients { id firstName lastName } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patients" count == 4

# Test 3: Query clinic information from Clinic A
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: patient-123
```json
{
  "query": "query GetClinicA { clinic(id: \"clinic-a\") { name address city state services } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.clinic.name" == "Downtown Medical Center"
jsonpath "$.data.clinic.city" == "San Francisco"
jsonpath "$.data.clinic.services" includes "General Practice"

# Test 4: Query doctors from Clinic A
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: doctor-789
```json
{
  "query": "query GetDoctorsA { doctors { id firstName lastName specialty clinicId } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.doctors[0].clinicId" == "clinic-a"
jsonpath "$.data.doctors" count == 3

# Test 5: Query appointments for a patient
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: patient-123
```json
{
  "query": "query GetPatientAppointments { patient(id: \"patient-1\") { appointments { id doctorId scheduledAt status reason } } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.appointments[0].status" exists
jsonpath "$.data.patient.appointments[*].status" includes "COMPLETED"

# Test 6: Query medical record for a patient
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: doctor-789
```json
{
  "query": "query GetMedicalRecord { patient(id: \"patient-1\") { medicalRecord { bloodType allergies currentMedications { name dosage } vitalSigns { heartRate bloodPressureSystolic } } } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.medicalRecord.bloodType" == "O+"
jsonpath "$.data.patient.medicalRecord.allergies" includes "Penicillin"
jsonpath "$.data.patient.medicalRecord.currentMedications[0].name" == "Lisinopril"
jsonpath "$.data.patient.medicalRecord.currentMedications[0].dosage" == "10mg"

# Test 7: Cross-subgraph query - Patient with appointments
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: patient-123
```json
{
  "query": "query PatientWithAppointments { patient(id: \"patient-1\") { firstName lastName appointments { scheduledAt doctor { firstName lastName specialty } status } } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.firstName" == "John"
jsonpath "$.data.patient.appointments[*].doctor.specialty" includes "General Practice"

# Test 8: Query multiple clinics (federation test)
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: patient-123
```json
{
  "query": "query GetClinics { clinics { id name address phone } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.clinics[0].name" exists

# Test 9: Query appointment with doctor details
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: patient-123
```json
{
  "query": "query GetAppointmentDetails { patient(id: \"patient-1\") { appointments { id scheduledAt duration status doctor { firstName lastName specialty email } } } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.appointments[0].duration" == 30
jsonpath "$.data.patient.appointments[0].doctor.firstName" == "Robert"

# Test 10: Test restricted field access (SSN should be inaccessible in federated schema)
POST http://localhost:4000/graphql
Content-Type: application/json
```json
{
  "query": "query { __type(name: \"Patient\") { fields { name } } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.__type.fields[?(@.name == 'ssn')]" isEmpty

# Test 11: Query medical record with vital signs
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: doctor-789
```json
{
  "query": "query GetVitalSigns { patient(id: \"patient-2\") { medicalRecord { vitalSigns { recordedAt heartRate temperature respiratoryRate } } } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.medicalRecord.vitalSigns[0].heartRate" == 68
jsonpath "$.data.patient.medicalRecord.vitalSigns[0].temperature" == 98.2

# Test 12: Test enum values
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: patient-123
```json
{
  "query": "query { __type(name: \"AppointmentStatus\") { enumValues { name } } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.__type.enumValues[*].name" includes "SCHEDULED"
jsonpath "$.data.__type.enumValues[*].name" includes "COMPLETED"
jsonpath "$.data.__type.enumValues[*].name" includes "CANCELLED"

################################################################################
# Dynamic Schema Contracts Tests - Different Access Levels
################################################################################

# Test 13: Public Access (no API key) - Should only see basic patient info
POST http://localhost:4000/graphql
Content-Type: application/json
```json
{
  "query": "{ patient(id: \"patient-1\") { id firstName lastName } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.id" == "patient-1"
jsonpath "$.data.patient.firstName" == "John"
jsonpath "$.data.patient.lastName" == "Doe"

# Test 14: Patient A Access - Can see own info and clinic A
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: patient-123
```json
{
  "query": "{ patient(id: \"patient-1\") { id firstName lastName email phone } clinic(id: \"clinic-a\") { name } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.email" == "john.doe@email.com"
jsonpath "$.data.patient.phone" == "555-0101"
jsonpath "$.data.clinic.name" == "Downtown Medical Center"

# Test 15: Patient A Access - Different patient (patient-2)
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: patient-456
```json
{
  "query": "{ patient(id: \"patient-2\") { id firstName lastName email } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.email" == "sarah.j@email.com"

# Test 16: Doctor A Access - Can see medical records and clinic A (nested)
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: doctor-789
```json
{
  "query": "{ patient(id: \"patient-1\") { id firstName medicalRecord { bloodType allergies } } doctors { firstName specialty } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.medicalRecord.bloodType" == "O+"
jsonpath "$.data.patient.medicalRecord.allergies" contains "Penicillin"
jsonpath "$.data.doctors[0].specialty" exists

# Test 17: Doctor A Access - Different query (nested)
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: doctor-321
```json
{
  "query": "{ patient(id: \"patient-2\") { id firstName medicalRecord { bloodType } } doctors { firstName lastName } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.medicalRecord.bloodType" == "A+"
jsonpath "$.data.doctors[0].firstName" == "Robert"

# Test 18: Billing Access - Can see insurance but NOT medical or clinic data
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: billing-555
```json
{
  "query": "{ patient(id: \"patient-1\") { id firstName insuranceProvider insurancePolicyNumber } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.insuranceProvider" == "HealthCare Plus"
jsonpath "$.data.patient.insurancePolicyNumber" == "HCP-789456"

# Test 19: Billing Access - Cannot see medical records (nested field doesn't exist)
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: billing-555
```json
{
  "query": "{ __type(name: \"Patient\") { fields { name } } }"
}
```
HTTP 200
[Asserts]
# medicalRecord field should not be available on Patient for billing role
jsonpath "$.data.__type.fields[?(@.name == 'medicalRecord')]" isEmpty

# Test 20: Admin Access - Can see everything except @inaccessible fields
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: admin-999
```json
{
  "query": "{ patient(id: \"patient-1\") { id firstName insuranceProvider } accessLog(patientId: \"patient-1\") { timestamp } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.insuranceProvider" == "HealthCare Plus"
jsonpath "$.data.accessLog" exists

# Test 21: Admin can access clinic-a
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: admin-999
```json
{
  "query": "{ clinicA: clinic(id: \"clinic-a\") { name } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.clinicA.name" == "Downtown Medical Center"

# Test 22: Introspection - Check what fields are visible for different roles
POST http://localhost:4000/graphql
Content-Type: application/json
```json
{
  "query": "{ __type(name: \"Patient\") { fields { name } } }"
}
```
HTTP 200
[Asserts]
# Public access should only see id, firstName, lastName
jsonpath "$.data.__type.fields[?(@.name == 'email')]" isEmpty
jsonpath "$.data.__type.fields[?(@.name == 'ssn')]" isEmpty

# Test 23: Patient can see their vital signs (nested)
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: patient-123
```json
{
  "query": "{ patient(id: \"patient-1\") { id medicalRecord { vitalSigns { heartRate temperature } } } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.medicalRecord.vitalSigns[0].heartRate" == 72
jsonpath "$.data.patient.medicalRecord.vitalSigns[0].temperature" == 98.6

# Test 24: Patient A can see their appointments at clinic A
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: patient-123
```json
{
  "query": "{ patient(id: \"patient-1\") { id appointments { id scheduledAt reason status } } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.appointments[0].reason" == "Annual checkup"
jsonpath "$.data.patient.appointments[0].status" == "SCHEDULED"

# Test 25: Verify patient cannot access currentMedications (doctor-access only)
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: patient-123
```json
{
  "query": "{ patient(id: \"patient-1\") { medicalRecord { currentMedications { name } } } }"
}
```
HTTP 200
[Asserts]
# Should get error because currentMedications is doctor-access only
jsonpath "$.errors[0].message" contains "does not have a field named 'currentMedications'"

# Test 26: Verify doctor CAN access currentMedications
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: doctor-789
```json
{
  "query": "{ patient(id: \"patient-1\") { medicalRecord { currentMedications { name dosage frequency } diagnoses { code description } } } }"
}
```
HTTP 200
[Asserts]
jsonpath "$.data.patient.medicalRecord.currentMedications[0].name" == "Lisinopril"
jsonpath "$.data.patient.medicalRecord.currentMedications[0].dosage" == "10mg"
jsonpath "$.data.patient.medicalRecord.diagnoses[0].code" == "I10"

# Test 27: Doctor can access clinic-b when x-clinic header is set to "b"
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: doctor-789
x-clinic: b
```json
{
  "query": "{ doctors { firstName lastName specialty } }"
}
```
HTTP 200
[Asserts]
# Should get clinic-b doctors (Maria Garcia, David Thompson, etc.)
jsonpath "$.data.doctors[0].firstName" == "Maria"
jsonpath "$.data.doctors[0].lastName" == "Garcia"
jsonpath "$.data.doctors[0].specialty" == "General Practice"

# Test 28: Default routing to clinic-a without x-clinic header
POST http://localhost:4000/graphql
Content-Type: application/json
x-api-key: doctor-789
```json
{
  "query": "{ doctors { firstName lastName specialty } }"
}
```
HTTP 200
[Asserts]
# Should get clinic-a doctors (Robert Smith, Lisa Anderson, etc.)
jsonpath "$.data.doctors[0].firstName" == "Robert"
jsonpath "$.data.doctors[0].lastName" == "Smith"
jsonpath "$.data.doctors[0].specialty" == "General Practice"