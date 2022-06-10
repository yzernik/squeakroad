import React, { useEffect } from 'react'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'
import { Link, withRouter } from 'react-router-dom'
import moment from 'moment'

import SentPaymentCard from './SentPaymentCard'
import Loader from '../../components/Loader'


import {
  fetchSentPayments,
  clearSentPayments,
  selectSentPayments,
  selectSentPaymentsStatus,
  selectLastSentPaymentsSqueak,
} from './paymentsSlice'


const SentPayments = (props) => {
  const sentPayments = useSelector(selectSentPayments);
  const sentPaymentsStatus = useSelector(selectSentPaymentsStatus);
  const lastSentPayment = useSelector(selectLastSentPaymentsSqueak);
  const dispatch = useDispatch();

  useEffect(() => {
    window.scrollTo(0, 0)
    console.log('fetchSentPayments');
    dispatch(clearSentPayments());
    dispatch(fetchSentPayments({
      squeakHash: props.squeakHash,
      pubkey: props.pubkey,
      network: props.network,
      host: props.host,
      port: props.port,
      limit: 10,
      lastSentPayment: null,
    }));
  }, [props.squeakHash])

  const fetchMore = () => {
    dispatch(fetchSentPayments({
      squeakHash: props.squeakHash,
      pubkey: props.pubkey,
      network: props.network,
      host: props.host,
      port: props.port,
      limit: 10,
      lastSentPayment: lastSentPayment,
    }));
  }

  const goToSqueak = (id) => {
    props.history.push(`/app/squeak/${id}`)
  }

  const renderedListItems = sentPayments.map(sentPayment=>{
    return <SentPaymentCard sentPayment={sentPayment} />
  })

  return <>
  {renderedListItems}

  {sentPaymentsStatus === 'loading' ?
    <div className="todo-list">
      <Loader />
    </div>
    :
    <div onClick={() => fetchMore()} className='squeak-btn-side squeak-btn-active'>
      LOAD MORE
    </div>
  }

  </>
}

export default withRouter(SentPayments)
