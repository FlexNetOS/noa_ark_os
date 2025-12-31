import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8787';
const WORKFLOW_BODY = JSON.stringify({
  workflow: 'load-test',
  metadata: {
    actor: 'automation',
    source: 'server.tests',
  },
});

export const options = {
  vus: Number(__ENV.K6_VUS || 20),
  duration: __ENV.K6_DURATION || '20s',
  thresholds: {
    http_req_duration: ['p(95)<100'],
    http_req_failed: ['rate<0.001'],
  },
};

export default function () {
  const res = http.post(`${BASE_URL}/ui/workflows`, WORKFLOW_BODY, {
    headers: { 'Content-Type': 'application/json' },
  });

  check(res, {
    'status is 200': (r) => r.status === 200,
    'body has workflow id': (r) => r.json('id') !== undefined,
  });

  sleep(0.1);
}
