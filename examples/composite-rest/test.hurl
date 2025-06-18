POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query Calls {
  calls {
    summary
  }
}
```

HTTP 200
{"data":{"calls":[{"summary":"Initial sales call with Acme Corp"},{"summary":"Follow-up call with TechStart Inc"},{"summary":"Product demo for Enterprise Solutions"}]}}

# With deals through derive & lookup
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query Calls {
  calls {
    summary
    deals {
      name
    }
  }
}
```

HTTP 200
{"data":{"calls":[{"summary":"Initial sales call with Acme Corp","deals":[{"name":"Acme Corp Enterprise License"},{"name":"Acme Corp Support Package"}]},{"summary":"Follow-up call with TechStart Inc","deals":[{"name":"TechStart Inc Starter Plan"}]},{"summary":"Product demo for Enterprise Solutions","deals":[{"name":"Acme Corp Enterprise License"},{"name":"Enterprise Solutions Premium"}]}]}}

POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query Calls {
  calls {
    summary
    deals {
      name
      orders {
        createdAt
        lineItems {
          product {
            name
          }
        }
      }
    }
  }
}
```

HTTP 200
{"data":{"calls":[{"summary":"Initial sales call with Acme Corp","deals":[{"name":"Acme Corp Enterprise License","orders":[{"createdAt":"2023-12-26T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}},{"product":{"name":"Integration Module"}}]},{"createdAt":"2024-01-05T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}}]}]},{"name":"Acme Corp Support Package","orders":[{"createdAt":"2024-01-15T10:30:00Z","lineItems":[{"product":{"name":"Support Package Gold"}},{"product":{"name":"Training Package"}}]}]}]},{"summary":"Follow-up call with TechStart Inc","deals":[{"name":"TechStart Inc Starter Plan","orders":[{"createdAt":"2024-01-01T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}}]}]}]},{"summary":"Product demo for Enterprise Solutions","deals":[{"name":"Acme Corp Enterprise License","orders":[{"createdAt":"2023-12-26T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}},{"product":{"name":"Integration Module"}}]},{"createdAt":"2024-01-05T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}}]}]},{"name":"Enterprise Solutions Premium","orders":[]}]}]}}

# With deals through require
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query Calls {
  calls {
    summary
    deals: deals2 {
      name
    }
  }
}
```

HTTP 200
{"data":{"calls":[{"summary":"Initial sales call with Acme Corp","deals":[{"name":"Acme Corp Enterprise License"},{"name":"Acme Corp Support Package"}]},{"summary":"Follow-up call with TechStart Inc","deals":[{"name":"TechStart Inc Starter Plan"}]},{"summary":"Product demo for Enterprise Solutions","deals":[{"name":"Acme Corp Enterprise License"},{"name":"Enterprise Solutions Premium"}]}]}}

POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query Calls {
  calls {
    summary
    deals: deals2 {
      name
      orders {
        createdAt
        lineItems {
          product {
            name
          }
        }
      }
    }
  }
}
```

HTTP 200
{"data":{"calls":[{"summary":"Initial sales call with Acme Corp","deals":[{"name":"Acme Corp Enterprise License","orders":[{"createdAt":"2023-12-26T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}},{"product":{"name":"Integration Module"}}]},{"createdAt":"2024-01-05T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}}]}]},{"name":"Acme Corp Support Package","orders":[{"createdAt":"2024-01-15T10:30:00Z","lineItems":[{"product":{"name":"Support Package Gold"}},{"product":{"name":"Training Package"}}]}]}]},{"summary":"Follow-up call with TechStart Inc","deals":[{"name":"TechStart Inc Starter Plan","orders":[{"createdAt":"2024-01-01T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}}]}]}]},{"summary":"Product demo for Enterprise Solutions","deals":[{"name":"Acme Corp Enterprise License","orders":[{"createdAt":"2023-12-26T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}},{"product":{"name":"Integration Module"}}]},{"createdAt":"2024-01-05T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}}]}]},{"name":"Enterprise Solutions Premium","orders":[]}]}]}}

# List all countries
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query ListCountries {
  listAllCountries {
    name
  }
}
```

HTTP 200
{"data":{"listAllCountries":[{"name":"Togolese Republic"},{"name":"Department of Mayotte"},{"name":"Georgia"},{"name":"Republic of Vanuatu"},{"name":"Kyrgyz Republic"},{"name":"Republic of Niger"},{"name":"People's Republic of China"},{"name":"Tuvalu"},{"name":"Union of the Comoros"},{"name":"Bosnia and Herzegovina"},{"name":"Kingdom of Bahrain"},{"name":"Federal Republic of Somalia"},{"name":"Collectivity of Saint Barthélemy"},{"name":"Republic of Latvia"},{"name":"Cayman Islands"},{"name":"Kingdom of the Netherlands"},{"name":"Kingdom of Lesotho"},{"name":"Bolivarian Republic of Venezuela"},{"name":"Republic of Kenya"},{"name":"Republic of Turkey"},{"name":"Republic of Fiji"},{"name":"Republic of Trinidad and Tobago"},{"name":"Republic of Honduras"},{"name":"Bailiwick of Jersey"},{"name":"Republic of Djibouti"},{"name":"Réunion Island"},{"name":"Kingdom of Eswatini"},{"name":"Republic of Tajikistan"},{"name":"Kingdom of Saudi Arabia"},{"name":"Bermuda"},{"name":"New Zealand"},{"name":"Republic of Belarus"},{"name":"Territory of Christmas Island"},{"name":"Republic of the Gambia"},{"name":"French Polynesia"},{"name":"Democratic Republic of the Congo"},{"name":"Republic of Costa Rica"},{"name":"Republic of Malawi"},{"name":"Republic of the Congo"},{"name":"Sultanate of Oman"},{"name":"Republic of Iraq"},{"name":"Niue"},{"name":"Republic of Senegal"},{"name":"Lebanese Republic"},{"name":"Republic of Angola"},{"name":"Islamic Republic of Iran"},{"name":"Republic of Ecuador"},{"name":"Lao People's Democratic Republic"},{"name":"Democratic Socialist Republic of Sri Lanka"},{"name":"Aruba"},{"name":"Democratic Republic of São Tomé and Príncipe"},{"name":"Grenada"},{"name":"Montserrat"},{"name":"Sahrawi Arab Democratic Republic"},{"name":"Republic of Guinea"},{"name":"Virgin Islands"},{"name":"Republic of Panama"},{"name":"Republic of Yemen"},{"name":"Republic of Estonia"},{"name":"Co-operative Republic of Guyana"},{"name":"Grand Duchy of Luxembourg"},{"name":"Republic of Namibia"},{"name":"Republic of Kosovo"},{"name":"New Caledonia"},{"name":"Saint Pierre and Miquelon"},{"name":"State of Libya"},{"name":"Republic of the Union of Myanmar"},{"name":"State of Israel"},{"name":"Republic of Mali"},{"name":"Argentine Republic"},{"name":"Guiana"},{"name":"Republic of Uganda"},{"name":"Malaysia"},{"name":"Republic of Zambia"},{"name":"Federated States of Micronesia"},{"name":"Slovak Republic"},{"name":"Republic of Burundi"},{"name":"Republic of Serbia"},{"name":"Turkmenistan"},{"name":"Republic of South Africa"},{"name":"Republic of Kazakhstan"},{"name":"Canada"},{"name":"Republic of Poland"},{"name":"Commonwealth of Puerto Rico"},{"name":"Territory of the Cocos (Keeling) Islands"},{"name":"Republic of San Marino"},{"name":"American Samoa"},{"name":"State of Qatar"},{"name":"Gibraltar"},{"name":"Republic of Croatia"},{"name":"Svalbard og Jan Mayen"},{"name":"Isle of Man"},{"name":"State of Palestine"},{"name":"Republic of Mauritius"},{"name":"Saint Martin"},{"name":"Republic of Sierra Leone"},{"name":"Federation of Saint Christopher and Nevis"},{"name":"Iceland"},{"name":"Heard Island and McDonald Islands"},{"name":"State of Kuwait"},{"name":"Republic of China (Taiwan)"},{"name":"Republic of Cameroon"},{"name":"Republic of El Salvador"},{"name":"Macao Special Administrative Region of the People's Republic of China"},{"name":"Republic of Cuba"},{"name":"Federative Republic of Brazil"},{"name":"Solomon Islands"},{"name":"Hellenic Republic"},{"name":"Kingdom of Norway"},{"name":"Jamaica"},{"name":"United Republic of Tanzania"},{"name":"Plurinational State of Bolivia"},{"name":"Republic of Cyprus"},{"name":"Mongolia"},{"name":"Principality of Monaco"},{"name":"Tunisian Republic"},{"name":"Territory of the French Southern and Antarctic Lands"},{"name":"Syrian Arab Republic"},{"name":"Republic of Chile"},{"name":"Republic of Azerbaijan"},{"name":"French Republic"},{"name":"Republic of Korea"},{"name":"Republic of Ghana"},{"name":"Kingdom of Denmark"},{"name":"Sint Maarten"},{"name":"Principality of Andorra"},{"name":"Country of Curaçao"},{"name":"Swiss Confederation"},{"name":"Republic of North Macedonia"},{"name":"Republic of South Sudan"},{"name":"Republic of Haiti"},{"name":"Republic of Peru"},{"name":"Republic of Guinea-Bissau"},{"name":"Czech Republic"},{"name":"Portuguese Republic"},{"name":"Gabonese Republic"},{"name":"United Mexican States"},{"name":"Italian Republic"},{"name":"Republic of Finland"},{"name":"Republic of Zimbabwe"},{"name":"Kingdom of Spain"},{"name":"Japan"},{"name":"Republic of Seychelles"},{"name":"Republic of Albania"},{"name":"Republic of Moldova"},{"name":"Republic of Guatemala"},{"name":"Federal Republic of Germany"},{"name":"Guadeloupe"},{"name":"Republic of Bulgaria"},{"name":"Republic of Paraguay"},{"name":"State of Eritrea"},{"name":"Republic of the Sudan"},{"name":"Republic of Equatorial Guinea"},{"name":"Republic of Colombia"},{"name":"Federal Republic of Nigeria"},{"name":"Montenegro"},{"name":"Federal Democratic Republic of Nepal"},{"name":"Nation of Brunei, Abode of Peace"},{"name":"Republic of Austria"},{"name":"Turks and Caicos Islands"},{"name":"Commonwealth of the Bahamas"},{"name":"United Kingdom of Great Britain and Northern Ireland"},{"name":"British Indian Ocean Territory"},{"name":"Republic of Liberia"},{"name":"Anguilla"},{"name":"Central African Republic"},{"name":"People's Democratic Republic of Algeria"},{"name":"Saint Lucia"},{"name":"Romania"},{"name":"United Arab Emirates"},{"name":"Kingdom of Morocco"},{"name":"Commonwealth of the Northern Mariana Islands"},{"name":"United States Minor Outlying Islands"},{"name":"Republic of Botswana"},{"name":"Republic of Lithuania"},{"name":"Bonaire, Sint Eustatius and Saba"},{"name":"Republic of Cabo Verde"},{"name":"United States of America"},{"name":"Commonwealth of Australia"},{"name":"Guam"},{"name":"Kingdom of Tonga"},{"name":"Islamic Republic of Afghanistan"},{"name":"Falkland Islands"},{"name":"Republic of Nauru"},{"name":"Cook Islands"},{"name":"Hungary"},{"name":"Islamic Republic of Mauritania"},{"name":"Republic of Slovenia"},{"name":"Republic of Chad"},{"name":"Vatican City State"},{"name":"Republic of Benin"},{"name":"Republic of Singapore"},{"name":"Republic of Malta"},{"name":"Saint Vincent and the Grenadines"},{"name":"Republic of the Marshall Islands"},{"name":"Kingdom of Thailand"},{"name":"Principality of Liechtenstein"},{"name":"Republic of Ireland"},{"name":"Republic of the Maldives"},{"name":"Socialist Republic of Vietnam"},{"name":"Republic of Armenia"},{"name":"Islamic Republic of Pakistan"},{"name":"Republic of Côte d'Ivoire"},{"name":"Russian Federation"},{"name":"Martinique"},{"name":"Bouvet Island"},{"name":"Democratic Republic of Timor-Leste"},{"name":"Republic of Nicaragua"},{"name":"Independent State of Papua New Guinea"},{"name":"Arab Republic of Egypt"},{"name":"Hashemite Kingdom of Jordan"},{"name":"Oriental Republic of Uruguay"},{"name":"Burkina Faso"},{"name":"Kingdom of Cambodia"},{"name":"Federal Democratic Republic of Ethiopia"},{"name":"Independent and Sovereign Republic of Kiribati"},{"name":"Kingdom of Bhutan"},{"name":"Dominican Republic"},{"name":"Republic of Suriname"},{"name":"People's Republic of Bangladesh"},{"name":"Barbados"},{"name":"Democratic People's Republic of Korea"},{"name":"Greenland"},{"name":"Kingdom of Belgium"},{"name":"Republic of Palau"},{"name":"Territory of Norfolk Island"},{"name":"South Georgia and the South Sandwich Islands"},{"name":"Republic of the Philippines"},{"name":"Republic of India"},{"name":"Republic of Indonesia"},{"name":"Virgin Islands of the United States"},{"name":"Commonwealth of Dominica"},{"name":"Republic of Rwanda"},{"name":"Republic of Madagascar"},{"name":"Hong Kong Special Administrative Region of the People's Republic of China"},{"name":"Independent State of Samoa"},{"name":"Faroe Islands"},{"name":"Pitcairn Group of Islands"},{"name":"Antigua and Barbuda"},{"name":"Åland Islands"},{"name":"Kingdom of Sweden"},{"name":"Republic of Mozambique"},{"name":"Republic of Uzbekistan"},{"name":"Antarctica"},{"name":"Territory of the Wallis and Futuna Islands"},{"name":"Ukraine"},{"name":"Belize"},{"name":"Saint Helena, Ascension and Tristan da Cunha"},{"name":"Bailiwick of Guernsey"},{"name":"Tokelau"}]}}

# Get a specific commune
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query GetCommune {
  commune(code: "75056") {
    code
    nom
    region {
      code
      nom
    }
    departement {
      code
      nom
    }
  }
}
```

HTTP 200
{"data":{"commune":{"code":"75056","nom":"Paris","region":{"code":"11","nom":"Île-de-France"},"departement":{"code":"75","nom":"Paris"}}}}
