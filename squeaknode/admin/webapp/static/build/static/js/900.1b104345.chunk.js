"use strict";(self.webpackChunktwitter_frontend=self.webpackChunktwitter_frontend||[]).push([[900],{2900:function(e,t,a){a.r(t),a.d(t,{default:function(){return N}});var n=a(885),l=a(2791),c=a(8619),i=a(9271),r=a(3652),m=a(8608),s=(a(2426),a(6238)),o=a(6440),u=a(8785),d=a(6789),p=a(9899),E=a(6515),f=a(6682),v=a(6617),k=a(1674),N=(0,i.EN)((function(e){var t=(0,l.useState)("Squeaks"),a=(0,n.Z)(t,2),i=a[0],N=a[1],y=(0,l.useState)(!1),b=(0,n.Z)(y,2),g=b[0],h=b[1],S=(0,l.useState)(""),q=(0,n.Z)(S,2),C=q[0],I=q[1],w=(0,l.useState)(!1),P=(0,n.Z)(w,2),x=P[0],T=P[1],Z=(0,l.useState)(!1),R=(0,n.Z)(Z,2),M=R[0],B=R[1],F=(0,l.useState)(!1),A=(0,n.Z)(F,2),H=A[0],K=A[1],L=(0,l.useState)(!1),D=(0,n.Z)(L,2),z=D[0],O=D[1],V=(0,l.useState)(!1),W=(0,n.Z)(V,2),_=W[0],J=W[1],U=(0,l.useState)(!1),X=(0,n.Z)(U,2),$=X[0],j=X[1],G=(0,l.useState)("Sent Payments"),Q=(0,n.Z)(G,2),Y=Q[0],ee=Q[1],te=(0,l.useState)(!1),ae=(0,n.Z)(te,2),ne=ae[0],le=ae[1],ce=e.match.params.username,ie=(0,d.v9)(E.pO),re=(0,d.v9)(E.JI),me=(0,d.v9)(E.Ht),se=(0,d.v9)(f.oW),oe=(0,d.v9)(p.KA),ue=(0,d.I0)();(0,l.useEffect)((function(){window.scrollTo(0,0),ue((0,p.In)(e.match.params.username)),we(),ue((0,f.Zb)({pubkey:e.match.params.username})),T(!1)}),[e.match.params.username]);var de=(0,l.useRef)(!0);(0,l.useEffect)((function(){de.current?de.current=!1:document.getElementsByTagName("body")[0].style.cssText=ne&&"overflow-y: hidden; margin-right: 17px"}),[ne]),(0,l.useEffect)((function(){return function(){return document.getElementsByTagName("body")[0].style.cssText=""}}),[]);var pe=function(e){N(e)},Ee=function(e){var t=e.values;ue((0,p.ZR)({profileId:oe.getProfileId(),profileName:t.name})),Ne()},fe=function(){var e={profileId:oe.getProfileId()};console.log("Delete user here"),ue((0,p.XM)(e)),be()},ve=function(){ue((0,p.F0)({profileId:oe.getProfileId()})).then(u.SI).then((function(e){console.log(e),I(e)}))},ke=function(t){var a=t.values;ue((0,p.vs)({pubkey:ce,profileName:a.name})).then((function(){ue((0,p.In)(e.match.params.username))})),Se()},Ne=function(e,t){h(!1),le(!ne),setTimeout((function(){T(!x)}),20)},ye=function(e,t){h(!1),le(!ne),setTimeout((function(){B(!M)}),20)},be=function(){h(!1),le(!ne),setTimeout((function(){K(!H)}),20)},ge=function(){h(!1),le(!ne),setTimeout((function(){O(!z)}),20)},he=function(e,t){le(!ne),t&&ee(t),t&&ee(t),setTimeout((function(){J(!_)}),20)},Se=function(e,t){le(!ne),setTimeout((function(){j(!$)}),20)},qe=function(e){e.stopPropagation()},Ce=function(e){if(null!=e){var t=new FileReader;t.addEventListener("load",(function(){var e=t.result.split(",")[1];Ie(e)}),!1),e&&t.readAsDataURL(e)}},Ie=function(e){ue((0,p.W$)({profileId:oe.getProfileId(),imageBase64:e}))},we=function(){ue((0,E.u8)()),ue((0,E.n2)({profilePubkey:e.match.params.username,limit:10,lastSqueak:null}))},Pe=function(){h(!g)},xe=function(){return l.createElement(o.l0,{onSubmit:ke,className:"Squeak-input-side"},l.createElement("div",{className:"edit-input-wrap"},l.createElement(o.II,{class:"informed-input",name:"name",label:"Profile Name",placeholder:"Satoshi"})),l.createElement("div",{className:"edit-input-wrap"},l.createElement(o.II,{class:"informed-input",name:"pubkey",label:"Public Key",defaultValue:ce,readOnly:!0,disabled:!0})),l.createElement("div",{className:"inner-input-links"},l.createElement("div",{className:"input-links-side"}),l.createElement("div",{className:"squeak-btn-holder"},l.createElement("div",{style:{fontSize:"13px",color:null}}),l.createElement("button",{type:"submit",className:"squeak-btn-side squeak-btn-active"},"Submit"))))},Te=function(){return l.createElement(o.l0,{onSubmit:Ee,className:"Squeak-input-side"},l.createElement("div",{className:"edit-input-wrap"},l.createElement(o.II,{class:"informed-input",name:"name",label:"Profile Name",placeholder:"Satoshi"})),l.createElement("div",{className:"inner-input-links"},l.createElement("div",{className:"input-links-side"}),l.createElement("div",{className:"squeak-btn-holder"},l.createElement("div",{style:{fontSize:"13px",color:null}}),l.createElement("button",{type:"submit",className:"squeak-btn-side squeak-btn-active"},"Submit"))))},Ze=function(){return l.createElement(o.l0,{onSubmit:Ce,className:"Squeak-input-side"},l.createElement("div",{className:"modal-profile-pic"},l.createElement("div",{className:"modal-back-pic"},l.createElement("img",{src:oe?"".concat((0,r.d)(oe)):null,alt:"profile"}),l.createElement("div",null,l.createElement(c.cn,null),l.createElement("input",{onChange:function(){return function(){var e=document.getElementById("avatar").files[0];Ce(e),ye()}()},title:" ",id:"avatar",style:{opacity:"0"},type:"file"})))))},Re=function(){return l.createElement(o.l0,{onSubmit:fe,className:"Squeak-input-side"},l.createElement("div",{className:"inner-input-links"},l.createElement("div",{className:"input-links-side"}),l.createElement("div",{className:"squeak-btn-holder"},l.createElement("div",{style:{fontSize:"13px",color:null}}),l.createElement("button",{type:"submit",className:"squeak-btn-side squeak-btn-active"},"Delete"))))},Me=function(){return l.createElement(o.l0,{onSubmit:ve,className:"Squeak-input-side"},l.createElement("div",{className:"edit-input-wrap"},l.createElement(o.II,{class:"informed-input",name:"privateKey",label:"Display Private Key",initialValue:C,readOnly:!0})),l.createElement("div",{className:"inner-input-links"},l.createElement("div",{className:"input-links-side"}),l.createElement("div",{className:"squeak-btn-holder"},l.createElement("div",{style:{fontSize:"13px",color:null}}),l.createElement("button",{type:"submit",className:"squeak-btn-side squeak-btn-active"},"Export"))))};return console.log(se),l.createElement("div",null,l.createElement("div",null,l.createElement("div",{className:"profile-wrapper"},l.createElement("div",{className:"profile-header-wrapper"},l.createElement("div",{className:"profile-header-back"},l.createElement("div",{onClick:function(){return window.history.back()},className:"header-back-wrapper"},l.createElement(c.r1,null))),l.createElement("div",{className:"profile-header-content"},l.createElement("div",{className:"profile-header-name"},ce))),l.createElement("div",{className:"profile-banner-wrapper"},l.createElement("img",{alt:""})),l.createElement("div",{className:"profile-details-wrapper"},l.createElement("div",{className:"profile-options"},l.createElement("div",{className:"profile-image-wrapper"},l.createElement("img",{src:oe?"".concat((0,r.d)(oe)):null,alt:""})),oe&&l.createElement("div",{id:"profileMoreMenu",onClick:Pe,className:"Nav-link"},l.createElement("div",{className:"Nav-item-hover"},l.createElement(c.nl,null)),l.createElement("div",{onClick:function(){return Pe()},style:{display:g?"block":"none"},className:"more-menu-background"},l.createElement("div",{className:"more-modal-wrapper"},g?l.createElement("div",{style:{top:document.getElementById("profileMoreMenu")&&"".concat(document.getElementById("profileMoreMenu").getBoundingClientRect().top-40,"px"),left:document.getElementById("profileMoreMenu")&&"".concat(document.getElementById("profileMoreMenu").getBoundingClientRect().left,"px"),height:"210px"},onClick:function(e){return function(e){e.stopPropagation()}(e)},className:"more-menu-content"},l.createElement("div",{onClick:Ne,className:"more-menu-item"},l.createElement("span",null,"Edit Profile")),l.createElement("div",{onClick:ye,className:"more-menu-item"},l.createElement("span",null,"Change Image")),oe.getHasPrivateKey()&&l.createElement("div",{onClick:ge,className:"more-menu-item"},l.createElement("span",null,"Export Private Key")),l.createElement("div",{onClick:be,className:"more-menu-item"},l.createElement("span",null,"Delete Profile"))):null))),oe&&l.createElement("div",{onClick:function(e){return oe.getFollowing()?function(e,t){console.log(e),e.stopPropagation(),ue((0,p.Kb)(t))}(e,oe.getProfileId()):function(e,t){e.stopPropagation(),ue((0,p.E9)(t))}(e,oe.getProfileId())},className:oe.getFollowing()?"unfollow-switch profile-edit-button":"profile-edit-button"},l.createElement("span",null,l.createElement("span",null,oe.getFollowing()?"Following":"Follow"))),!oe&&l.createElement("div",{onClick:function(e){return Se("create")},className:"profiles-create-button"},l.createElement("span",null,"Add Contact Profile"))),l.createElement("div",{className:"profile-details-box"},l.createElement("div",{className:"profile-name"},oe?oe.getProfileName():""),l.createElement("div",{className:"profile-username"},"@",ce),l.createElement("div",{className:"profile-info-box"},"\xa0")),l.createElement("div",{className:"profile-social-box"},l.createElement("div",{onClick:function(){return he(0,"Sent Payments")}},l.createElement("p",{className:"follow-num"}," ",se&&se.getAmountSpentMsat()/1e3," "),l.createElement("p",{className:"follow-text"}," sats spent ")),l.createElement("div",{onClick:function(){return he(0,"Received Payments")}},l.createElement("p",{className:"follow-num"}," ",se&&se.getAmountEarnedMsat()/1e3," "),l.createElement("p",{className:"follow-text"}," sats earned ")))),l.createElement("div",{className:"profile-nav-menu"},l.createElement("div",{key:"squeaks",onClick:function(){return pe("Squeaks")},className:"Squeaks"===i?"profile-nav-item activeTab":"profile-nav-item"},"Squeaks"),l.createElement("div",{key:"replies",onClick:function(){return pe("Squeaks&Replies")},className:"Squeaks&Replies"===i?"profile-nav-item activeTab":"profile-nav-item"},"Squeaks & replies"),l.createElement("div",{key:"liked",onClick:function(){return pe("Liked")},className:"Liked"===i?"profile-nav-item activeTab":"profile-nav-item"},"Liked")),"Squeaks"===i?ie.map((function(e){if(!e.getReplyTo())return l.createElement(s.Z,{squeak:e,key:e.getSqueakHash(),id:e.getSqueakHash(),user:e.getAuthor()})})):"Squeaks&Replies"===i?ie.map((function(e){return l.createElement(s.Z,{squeak:e,key:e.getSqueakHash(),id:e.getSqueakHash(),user:e.getAuthor()})})):"Liked"===i?ie.map((function(e){if(e.getLikedTimeMs())return l.createElement(s.Z,{squeak:e,key:e.getSqueakHash(),id:e.getSqueakHash(),user:e.getAuthor()})})):null,ie.length>0&&l.createElement(l.Fragment,null,"loading"==me?l.createElement(m.Z,null):l.createElement("div",{onClick:function(){return function(){var t;null==(t=ie)||0===t.length||t.slice(-1)[0],ue((0,E.n2)({profilePubkey:e.match.params.username,limit:10,lastSqueak:re}))}()},className:"squeak-btn-side squeak-btn-active"},"Load more"))),l.createElement("div",{onClick:function(){return Ne()},style:{display:x?"block":"none"},className:"modal-edit"},l.createElement("div",{onClick:function(e){return qe(e)},className:"modal-content"},l.createElement("div",{className:"modal-header"},l.createElement("div",{className:"modal-closeIcon"},l.createElement("div",{onClick:function(){return Ne()},className:"modal-closeIcon-wrap"},l.createElement(c.q3,null))),l.createElement("p",{className:"modal-title"},"Edit Profile")),l.createElement("div",{className:"modal-body"},l.createElement(Te,null)))),l.createElement("div",{onClick:function(){return ye()},style:{display:M?"block":"none"},className:"modal-edit"},l.createElement("div",{onClick:function(e){return qe(e)},className:"modal-content"},l.createElement("div",{className:"modal-header"},l.createElement("div",{className:"modal-closeIcon"},l.createElement("div",{onClick:function(){return ye()},className:"modal-closeIcon-wrap"},l.createElement(c.q3,null))),l.createElement("p",{className:"modal-title"},"Change Profile Image")),l.createElement("div",{className:"modal-body"},l.createElement("div",{className:"modal-banner"}),l.createElement(Ze,null)))),l.createElement("div",{onClick:function(){return be()},style:{display:H?"block":"none"},className:"modal-edit"},l.createElement("div",{onClick:function(e){return qe(e)},className:"modal-content"},l.createElement("div",{className:"modal-header"},l.createElement("div",{className:"modal-closeIcon"},l.createElement("div",{onClick:function(){return be()},className:"modal-closeIcon-wrap"},l.createElement(c.q3,null))),l.createElement("p",{className:"modal-title"},"Delete Profile")),l.createElement("div",{className:"modal-body"},l.createElement(Re,null)))),l.createElement("div",{onClick:function(){return ge()},style:{display:z?"block":"none"},className:"modal-edit"},l.createElement("div",{onClick:function(e){return qe(e)},className:"modal-content"},l.createElement("div",{className:"modal-header"},l.createElement("div",{className:"modal-closeIcon"},l.createElement("div",{onClick:function(){return ge()},className:"modal-closeIcon-wrap"},l.createElement(c.q3,null))),l.createElement("p",{className:"modal-title"},"Export Private Key")),l.createElement("div",{className:"modal-body"},l.createElement(Me,null)))),l.createElement("div",{onClick:function(){return he()},style:{display:_?"block":"none"},className:"modal-edit"},l.createElement("div",{onClick:function(e){return qe(e)},className:"modal-content"},l.createElement("div",{className:"modal-header no-b-border"},l.createElement("div",{className:"modal-closeIcon"},l.createElement("div",{onClick:function(){return he()},className:"modal-closeIcon-wrap"},l.createElement(c.q3,null))),l.createElement("p",{className:"modal-title"},null)),l.createElement("div",{className:"modal-body"},l.createElement("div",{className:"explore-nav-menu"},l.createElement("div",{onClick:function(){return ee("Sent Payments")},className:"Sent Payments"==Y?"explore-nav-item activeTab":"explore-nav-item"},"Sent Payments"),l.createElement("div",{onClick:function(){return ee("Received Payments")},className:"Received Payments"==Y?"explore-nav-item activeTab":"explore-nav-item"},"Received Payments")),l.createElement("div",{className:"modal-scroll"},"Sent Payments"===Y?l.createElement(l.Fragment,null,l.createElement(v.Z,{pubkey:e.match.params.username})):"Received Payments"===Y?l.createElement(l.Fragment,null,l.createElement(k.Z,{pubkey:e.match.params.username})):l.createElement("div",{className:"try-searching"},"Nothing to see here ..",l.createElement("div",null),"Try searching for people, usernames, or keywords"))))),l.createElement("div",{onClick:function(){return Se()},style:{display:$?"block":"none"},className:"modal-edit"},l.createElement("div",{onClick:function(e){return qe(e)},className:"modal-content"},l.createElement("div",{className:"modal-header"},l.createElement("div",{className:"modal-closeIcon"},l.createElement("div",{onClick:function(){return Se()},className:"modal-closeIcon-wrap"},l.createElement(c.q3,null))),l.createElement("p",{className:"modal-title"},"Add Contact Profile")),l.createElement("div",{className:"modal-body"},l.createElement(xe,null))))))}))}}]);
//# sourceMappingURL=900.1b104345.chunk.js.map