import { redirect } from 'next/navigation';

export default function Home() {
  // Manda automaticamente gli utenti alla pagina di login
  redirect('/login');
}