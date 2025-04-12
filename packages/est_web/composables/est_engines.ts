export default function useEstEngines() {
  return useFetch<{
    engines: string[]
  }>('/api/experimental/engines', {
    transform(data) {
      return {
        engines: data.engines.filter(engine => engine && !engine.endsWith("_")).sort((a, b) => a.length - b.length)
      };
    },
    onResponseError({ response }) {
      console.error('Error fetching engines: ', response.status, response.statusText);
      return {
        engines: [],
      };
    },
    onRequestError({ error }) {
      console.error('Error fetching engines: ', error);
      return {
        engines: [],
      }
    }
  });
}
