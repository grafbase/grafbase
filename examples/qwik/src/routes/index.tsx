import { component$, useServerMount$, useSignal } from '@builder.io/qwik';
import { grafbaseClient } from '~/utils/grafbase';

interface Plants {
  plantCollection: {
    edges: Plant[]
  }
}

interface Plant {
  id: string
  name: string
  description: string
}

export const GetAllPlantsQuery = `
  query GetAllPlants($first: Int!) {
    plantCollection(first: $first) {
      edges {
        node {
          id
          name
          description
        }
      }
    }
  }
`

export const AddNewPlantMutation = `
  mutation AddNewPlant($name: String!, $description: String!) {
    plantCreate(input: { name: $name, description: $description }) {
      plant {
        id
        name
        description
      }
    }
  }
`

export default component$(() => {
  const newPlant = useSignal('');
  const newPlantDescription = useSignal('');
  const allPlants = useSignal<Plants>();

  useServerMount$(async () => {
    const plants: Plants = await grafbaseClient({ query: GetAllPlantsQuery, variables: {first: 100}});
    allPlants.value = plants;
  })

  return (
    <div>
      <h1>Plants</h1>
      { allPlants.value?.plantCollection.edges.map(({node}) => (
        <>
        <div>{node?.name} : {node?.description}</div>
        </>
      ))}

      <h2>New plant</h2>

      <input id="name" name="name" placeholder="Name" value={newPlant.value} onInput$={ event =>
      newPlant.value = (event.target as HTMLInputElement).value } />
      <br />

      <input id="description" name="description" placeholder="Describe the plant" value={newPlantDescription.value} onInput$={ event =>
      newPlantDescription.value = (event.target as HTMLInputElement).value } />
      <br />
      
      <button
        onClick$={async () => {
          await grafbaseClient({ query: AddNewPlantMutation, variables: { name: newPlant.value, description: newPlantDescription.value }})
          allPlants.value = await grafbaseClient({ query: GetAllPlantsQuery, variables: {first: 100}});
        }}
      > Add Plant
      </button>
    </div>
  );
});